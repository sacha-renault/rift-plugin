# Audio accumulator

## The problem

Audio plugins run on two threads that can't talk to each other easily:

- The **audio thread** is real-time. No locks, no allocations, no blocking. The host calls you with a buffer of samples and you better be done before the next one arrives.
- The **UI thread** draws things. It needs audio data to display waveforms, spectrums, peak meters but it can't just reach into the audio thread and grab it.

So we need a way to get samples from one side to the other without either thread waiting on the other.

## How it works

```text
Audio thread                                          UI thread
───────────                                           ─────────

Host hands us                                         On each frame
&[f32] per channel                                    (e.g. 60fps)
       │                                                    │
       ▼                                                    ▼
┌──────────────────┐                              ┌─────────────────────┐
│ AudioAccumulator │                              │ AudioAccumulator    │
│                  │    crossbeam queue           │   .dispatch()       │
│ Chops input into ├─────────────────────────────►│                     │
│ fixed-size blocks│    lock-free, no alloc       │ Pops all blocks,    │
│ per channel      │    per-channel ring          │ feeds them into:    │
└──────────────────┘                              └────────┬────────────┘
                                                           │
                                                           ▼
                                                  ┌────────────────────┐
                                                  │ ConsumerDispatcher │
                                                  │                    │
                                                  │ Routes each block  │
                                                  │ by consumer type   │
                                                  └──┬────┬─────────┬──┘
                                                     │    │         │
                            ┌────────────────────────┘    │         └────────────────┐
                            ▼                             ▼                          ▼
                      Averaged                     All (multi)               Channel(n)
                            │                             │                          │
                    Accumulates all channels        Forwards every            Forwards only
                    into intermediate buffer,       channel as-is,            channel n,
                    divides, sends mono result      with ChannelsInfo         ignores the rest
                    after last channel                    │                          │
                            │                             │                          │
                            ▼                             ▼                          ▼
                      ┌──────────────┐           ┌──────────────────┐         ┌──────────────────┐
                      │ MonoConsumer │           │  MultiConsumer   │         │  MonoConsumer or │
                      │MultiConsumer │           │       ...        │         │   MultiConsumer  │
                      └──────────────┘           └──────────────────┘         └──────────────────┘
```

## AudioAccumulator

Lives on both sides of the fence.

On the **audio thread**, the host gives us variable-length buffers (could be 32 samples, could be 4096). The accumulator chops these into fixed-size blocks of `N` samples (compile-time const generic) and pushes them into a per-channel crossbeam `ArrayQueue`. This is a bounded, lock-free, single-producer ring buffer. No allocations happen here, everything was pre-allocated at construction.

On the **UI thread**, `dispatch()` pops every available block and hands each one to the `ConsumerDispatcher`. If a channel's queue is empty mid-dispatch (shouldn't happen in normal operation, but can if the audio thread falls behind), the remaining data is discarded to stay in sync.

```rust
// Audio thread, called by the host
accumulator.push_slices(&mut [left.as_slice(), right.as_slice()].into_iter(), Some(block_info));

// UI thread, called every frame
accumulator.dispatch(&mut dispatcher);
```

## ConsumerDispatcher

This is where channel routing lives. Instead of every consumer implementing its own "am I looking at the right channel?" logic, the dispatcher handles it once.

There are two consumer traits:

- **`MonoConsumer`** - receives a plain `&[f32]` block with no channel context. Used for consumers that only need a single signal (averaged mix or a specific channel).
- **`MultiConsumer`** - receives `&[f32]` along with `ChannelsInfo`, so it knows which channel it's looking at and how many there are in total. Used for per-channel work like peak meters. Every `MultiConsumer` automatically implements `MonoConsumer` via a blanket impl, so it can be used anywhere a `MonoConsumer` is expected.

You register consumers with the appropriate method:

```rust
dispatcher.add_consumer_averaged(my_waveform.clone());    // MonoConsumer, gets mono mix
dispatcher.add_consumer_all(my_peak_meter.clone());       // MultiConsumer, gets every channel separately
dispatcher.add_consumer_at_channel(my_fft.clone(), 0);    // MonoConsumer, gets only channel 0
```

During `dispatch()`, the accumulator pops blocks channel by channel (ch0, ch1, ..., then next block set). For each block, the dispatcher:

1. If any consumer wants averaging: accumulates samples into a scratch buffer, dividing by total channels as it goes
2. Forwards the block to registered consumers:
   - **Averaged** → waits until the last channel, then sends the averaged buffer to `MonoConsumer`s
   - **All** → sends every channel immediately with full `ChannelsInfo` to `MultiConsumer`s
   - **Channel(n)** → sends only when `current == n`, raw block as-is to `MonoConsumer`s

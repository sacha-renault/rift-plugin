# Buffer Iterator Benchmark

Compares two ways to iterate over audio buffers: **sample-by-sample** (interleaved across channels) vs **channel-by-channel** (contiguous per channel).

## TL;DR

Channel iteration is faster in microbenchmarks, but the difference is irrelevant in practice. Both approaches use well under 0.1% of the real-time budget at 48kHz. Sample iteration should be the default, it's more ergonomic and naturally supports sample-accurate automation, cross-channel processing, and stateful DSP.

## Usage

```bash
cargo run -p buffer-iter-benchmark --release -- --samples 40000 --channels 2 --iterations 1000
```

## Results

Measured on a single machine, 2 channels, 40000 samples/buffer, 1000 iterations, release mode.

| Test | Sample (µs/buf) | Channel (µs/buf) | Speedup | Budget % (sample) |
|---|---|---|---|---|
| Gain (×0.5) | 39.5 | 7.1 | 5.60× | 0.005% |
| Soft clipping (tanh) | 396.5 | 369.9 | 1.07× | 0.048% |
| Hard clipping (clamp) | 51.9 | 8.9 | 5.83× | 0.006% |
| Complex ops (sin+cos) | 659.4 | 608.7 | 1.08× | 0.079% |
| Branching ops | 78.5 | 16.4 | 4.78× | 0.009% |
| Atomic reads | 44.6 | 24.2 | 1.84× | 0.005% |

Budget % is the fraction of the available real-time budget at 48kHz (a 40000-sample buffer gives ~833ms of real time).

## Analysis

Channel iteration wins big on trivial ops (gain, clamp) because the compiler can autovectorize a tight loop over a contiguous slice. The moment the per-sample operation is non-trivial, `tanh`, `sin`, `cos`, anything the compiler can't turn into SIMD, the advantage drops to single-digit percentages.

Real-world DSP is almost never a simple multiply. Filters carry state across samples. Envelope followers branch. Oscillators compute trig. Parameter smoothing needs per-sample interpolation. These are the operations where channel vs sample iteration barely matters.

More importantly, even the *slower* path (sample iteration) never exceeds 0.08% of the buffer budget. You could stack hundreds of plugins using sample iteration before approaching any real-time constraint. The performance difference between the two approaches is not a bottleneck, it is noise in the context of a full signal chain.

## Why sample iteration is the better default

Channel iteration forces you to process one channel at a time. This makes cross-channel work (stereo panning, mid/side encoding, channel linking) awkward, you end up manually indexing into other channels and tracking sample positions yourself, which defeats the purpose of having an iterator.

Sample iteration gives you all channels at a given time step. This is the natural shape for:

- Sample-accurate parameter automation
- Stateful filters (biquads, one-poles, feedback delays)
- Cross-channel processing (stereo width, balance, M/S)
- Any operation where you need to know "what sample am I on"

Channel iteration is available on `Buffers` for the rare cases where you genuinely process each channel independently with no shared state, bulk gain, normalization, or offline analysis.
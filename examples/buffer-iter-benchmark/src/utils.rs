use std::time::{Duration, Instant};

use rift_plugin::prelude::Buffer;

pub struct AudioBuffer {
    channels: Vec<Vec<f32>>,
    ptrs: Vec<*mut f32>,
}

impl AudioBuffer {
    pub fn new(num_channels: usize, samples: usize) -> Self {
        let mut channels = Vec::with_capacity(num_channels);

        for channel_idx in 0..num_channels {
            let mut channel = vec![0.0f32; samples];
            let frequency = 440.0 * (channel_idx + 1) as f32;
            for i in 0..samples {
                let t = i as f32 / samples as f32;
                channel[i] = (t * 2.0 * std::f32::consts::PI * frequency).sin() * 0.5;
            }
            channels.push(channel);
        }

        let ptrs = channels.iter_mut().map(|ch| ch.as_mut_ptr()).collect();
        Self { channels, ptrs }
    }

    pub fn get_ptrs(&mut self) -> &[*mut f32] {
        for (i, channel) in self.channels.iter_mut().enumerate() {
            self.ptrs[i] = channel.as_mut_ptr();
        }
        &self.ptrs
    }
}

pub struct BenchResult {
    pub duration: Duration,
    pub samples: usize,
    pub num_channels: usize,
    pub iterations: usize,
}

impl BenchResult {
    pub fn throughput(&self) -> f64 {
        let total_samples = self.samples * self.num_channels * self.iterations;
        total_samples as f64 / self.duration.as_secs_f64()
    }

    pub fn print(&self, label: &str) {
        let total_samples = self.samples * self.num_channels * self.iterations;
        let seconds = self.duration.as_secs_f64();
        let duration_per_buffer = seconds / self.iterations as f64;
        let time_per_sample = seconds / total_samples as f64;
        let samples_per_second = self.throughput();

        println!("{label} Iterator");
        println!(
            "  Duration per buffer: {}",
            format_duration(duration_per_buffer)
        );

        let budget = self.samples as f64 / 48_000.0;
        let actual = self.duration.as_secs_f64() / self.iterations as f64;
        let percent = (actual / budget) * 100.0;
        println!("  Buffer time % (at 48kHz): {:.5}%", percent);
        println!(
            "  Throughput: {:.2} M samples/sec",
            samples_per_second / 1_000_000.0
        );
        println!("  Time per sample: {}", format_duration(time_per_sample));
    }
}

pub fn format_duration(secs: f64) -> String {
    if secs >= 1.0 {
        format!("{:.3} s", secs)
    } else if secs >= 0.001 {
        format!("{:.3} ms", secs * 1_000.0)
    } else if secs >= 0.000_001 {
        format!("{:.3} µs", secs * 1_000_000.0)
    } else {
        format!("{:.3} ns", secs * 1_000_000_000.0)
    }
}

pub fn benchmark<F>(
    factory: impl Fn(usize, usize) -> AudioBuffer,
    samples: usize,
    num_channels: usize,
    iterations: usize,
    process_fn: F,
) where
    F: Fn(&mut f32),
{
    // Sample iterator
    let sample_result = {
        let mut duration = Duration::ZERO;
        for _ in 0..iterations {
            let mut audio = factory(num_channels, samples);
            let mut buffer = Buffer::from_raw(audio.get_ptrs(), samples as u32);

            let start = Instant::now();
            for v in buffer.iter_samples() {
                for sample in v {
                    process_fn(sample);
                }
            }
            duration += start.elapsed();
        }
        BenchResult {
            duration,
            samples,
            num_channels,
            iterations,
        }
    };

    // Channel iterator
    let channel_result = {
        let mut duration = Duration::ZERO;
        for _ in 0..iterations {
            let mut audio = factory(num_channels, samples);
            let mut buffer = Buffer::from_raw(audio.get_ptrs(), samples as u32);

            let start = Instant::now();
            for channel in buffer.iter_channels_mut() {
                for sample in channel.iter_mut() {
                    process_fn(sample);
                }
            }
            duration += start.elapsed();
        }
        BenchResult {
            duration,
            samples,
            num_channels,
            iterations,
        }
    };

    sample_result.print("SAMPLE");
    channel_result.print("CHANNEL");
    println!(
        "  => Channel is {:.2}x faster\n",
        channel_result.throughput() / sample_result.throughput()
    );
}

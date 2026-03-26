mod utils;

use std::sync::{Arc, atomic::AtomicU32};

use clap::Parser;

use utils::*;

#[derive(Parser, Debug)]
#[command(name = "buffer-bench")]
struct Args {
    /// Number of samples per buffer
    #[arg(short, long, default_value_t = 512)]
    samples: usize,

    /// Number of audio channels
    #[arg(short, long, default_value_t = 2)]
    channels: usize,

    /// Number of iterations per test
    #[arg(short, long, default_value_t = 1000)]
    iterations: usize,
}

macro_rules! run_test {
    ($name:expr, $factory:expr, $samples:expr, $channels:expr, $iterations:expr, $f:expr) => {
        println!("--- Test {} ---", $name);
        benchmark($factory, $samples, $channels, $iterations, $f);
    };
}

fn main() {
    let args = Args::parse();
    let factory = |ch, s| AudioBuffer::new(ch, s);
    let Args {
        samples,
        channels,
        iterations,
    } = args;

    println!("=== BENCHMARK: Sample Iterator vs Channel Iterator ===");
    println!(
        "Samples: {}, Channels: {}, Iterations: {}, Total: {} M samples\n",
        samples,
        channels,
        iterations,
        (samples * channels * iterations) / 1_000_000
    );

    run_test!(
        "Gain (x0.5)",
        factory,
        samples,
        channels,
        iterations,
        |s: &mut f32| *s *= 0.5
    );
    run_test!(
        "Soft Clipping (tanh)",
        factory,
        samples,
        channels,
        iterations,
        |s: &mut f32| *s = s.tanh()
    );
    run_test!(
        "Hard Clipping",
        factory,
        samples,
        channels,
        iterations,
        |s: &mut f32| *s = s.clamp(-1.0, 1.0)
    );
    run_test!(
        "Complex ops (sin + cos)",
        factory,
        samples,
        channels,
        iterations,
        |s: &mut f32| *s = (*s * 2.0).sin() * (*s * 3.0).cos()
    );
    run_test!(
        "Branching ops",
        factory,
        samples,
        channels,
        iterations,
        |s: &mut f32| *s = if s.abs() > 0.5 { s.sqrt() } else { *s }
    );

    let v = Arc::new(AtomicU32::new(0f32.to_bits()));
    run_test!(
        "Atomic reads",
        factory,
        samples,
        channels,
        iterations,
        move |s: &mut f32| { *s += f32::from_bits(v.load(std::sync::atomic::Ordering::Relaxed)) }
    );
}

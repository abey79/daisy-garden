use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;

use clap::{Parser, ValueEnum};

use dg_noise::NoiseGenerator;

#[derive(Parser)]
#[command(name = "noise-gen")]
#[command(about = "Generate noise samples and save to text file", long_about = None)]
struct Cli {
    /// Output file path
    #[arg(short, long)]
    output: PathBuf,

    #[arg(short, long, default_value_t = 44100)]
    sample_rate: u64,

    /// Number of samples to generate
    #[arg(short, long, default_value_t = 1000000)]
    num_samples: usize,

    /// Type of noise to generate
    #[arg(short = 't', long, value_enum, default_value_t = NoiseType::White)]
    noise_type: NoiseType,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum NoiseType {
    White,
    Red, //TODO: add moar
}

impl NoiseType {
    fn build(&self, sample_rate: u64) -> Box<dyn NoiseGenerator> {
        let mut rng = rand::thread_rng();
        match self {
            NoiseType::White => {
                Box::new(dg_noise::WhiteNoiseGenerator::new_simple_from_rng(&mut rng))
            }

            NoiseType::Red => Box::new(dg_noise::RedNoiseGenerator::new_simple_from_rng(
                &mut rng,
                sample_rate,
            )),
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let mut generator = cli.noise_type.build(cli.sample_rate);

    let file = File::create(&cli.output)?;

    let mut writer = BufWriter::new(file);

    // Generate and write samples
    for _ in 0..cli.num_samples {
        let sample = generator.sample();

        writeln!(writer, "{}", sample)?;
    }

    // Ensure all data is written
    writer.flush()?;

    Ok(())
}

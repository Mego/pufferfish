use std::{fs::read_to_string, path::PathBuf};

use pufferfish::program::Program;

use clap::{Args, Parser};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(flatten)]
    input: Input,
}

#[derive(Args)]
#[group(required = true, multiple = false)]
struct Input {
    /// The file containing the program
    #[arg(short, long, value_name = "FILE")]
    file: Option<PathBuf>,

    /// The program
    code: Option<String>,
}

fn main() -> Result<(), anyhow::Error> {
    let cli = Cli::parse();
    let code = if let Some(input_file) = cli.input.file {
        read_to_string(input_file)?
    } else {
        cli.input.code.unwrap()
    };
    let mut program = Program::new(&code)?;
    loop {
        program.step();
    }
}

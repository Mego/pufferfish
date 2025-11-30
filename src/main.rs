use std::env::args;

use pufferfish::program::Program;

fn main() -> Result<(), anyhow::Error> {
    let code = args().nth(1).unwrap();
    let mut program = Program::new(&code)?;
    loop {
        program.step();
    }
}

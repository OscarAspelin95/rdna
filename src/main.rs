mod engine;
mod types;

use engine::{cleanup_terminal, run, setup_terminal};
use std::io::{self, stdout};

fn main() -> io::Result<()> {
    let mut stdout = stdout();
    setup_terminal(&mut stdout)?;

    let result = run(&mut stdout);
    cleanup_terminal(&mut stdout)?;
    result
}

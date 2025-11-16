mod editor;
mod rope;
mod tui;

use std::io::{self, Write};

use editor::Editor;
use tui::start_tui;

fn main() {
    let mut editor = Editor::new();

    println!("Launching TUI editor... (press q to quit)");

    if let Err(e) = start_tui(&mut editor) {
        eprintln!("Error: {}", e);
    }
}

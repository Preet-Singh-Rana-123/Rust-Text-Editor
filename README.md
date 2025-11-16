# Rust Text Editor

A fast, minimal terminal-based text editor built in Rust, featuring a custom Rope data structure for efficient text manipulation and a Crossterm-powered TUI.

---

## Features

### Efficient Editing with Rope
- Large files handled smoothly using a custom Rope implementation.

### Terminal UI (TUI)
- Fully interactive interface built with crossterm.

### Keyboard Input Support
- Insert characters, navigate, and exit using simple keybindings.

### Open Text Files
- Loads files like example.txt at startup.

### Modular Codebase
- Clean separation between editor logic, rope structure, and TUI rendering.

---

## Project Structure
```
├── Cargo.lock
├── Cargo.toml
├── example.txt                 # Sample file to load/edit
└── src
    ├── editor.rs               # Core editor logic (cursor, buffer updates)
    ├── rope.rs                 # Rope data structure implementation
    ├── tui.rs                  # Crossterm-based terminal UI
    └── main.rs                 # Entry point
```
---

## Installation & Setup
### 1. Clone the repository
```
 git clone https://github.com/your-username/rust-text-editor.git
 cd rust-text-editor
```

### 2. Run the project
```
cargo run
```

### If no file is supplied, the editor loads a blank buffer.

---

## How It Works
### Rope Data Structure
- Splits text into chunks for O(log n) inserts/deletes.
- Ideal for editors handling large files.
### Crossterm TUI
- Renders editor view
- Handles keyboard events in raw mode
- Manages cursor position and screen refresh

---

## Future Enhancements
- Syntax highlighting
- Search (Ctrl+F)
- Config file for themes and keybindings

---

## Contributing
- Pull requests are welcome!
- If you have ideas for improvements, feel free to open an issue.

---

## Author
- Preet Rana
- [preetrana1263@gmail.com]
- [www.linkedin.com/in/preet-rana-64235b292]

---

## License
- This project is open-sourced under the MIT License.

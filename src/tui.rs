use std::io::{Stdout, Write, stdout};

use crossterm::{
    cursor::{self, Hide, MoveTo, RestorePosition, SavePosition, Show},
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{self, Clear, ClearType},
};

use crate::editor::Editor;

pub fn start_tui(editor: &mut Editor) -> std::io::Result<()> {
    terminal::enable_raw_mode()?;

    let mut stdout = stdout();
    execute!(stdout, Hide)?;

    loop {
        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    // EXIT
                    KeyCode::Char('q') if key_event.modifiers == KeyModifiers::CONTROL => {
                        terminal::disable_raw_mode()?;
                        execute!(stdout, Show)?;
                        return Ok(());
                    }

                    // SAVE
                    KeyCode::Char('a') if key_event.modifiers == KeyModifiers::CONTROL => {
                        if editor.filename.is_none() {
                            let name = prompt(&mut stdout, "Save as")?;
                            editor.save_as(&name)?;
                        } else {
                            editor.save()?;
                        }
                    }

                    KeyCode::Char('o') if key_event.modifiers == KeyModifiers::CONTROL => {
                        let name = prompt(&mut stdout, "Open file")?;
                        if let Err(e) = editor.open_file(&name) {
                            let msg = format!("Failed to open {}: {}", name, e);
                            prompt(&mut stdout, &msg)?;
                        }
                    }

                    // CHARACTER INPUT
                    KeyCode::Char(c) => {
                        if key_event.modifiers == KeyModifiers::CONTROL {
                            match c {
                                'z' => editor.undo(),
                                'y' => editor.redo(),
                                _ => {}
                            }
                        } else {
                            editor.insert_at_cursor(&c.to_string());
                        }
                    }

                    // ENTER / BACKSPACE
                    KeyCode::Enter => editor.insert_at_cursor("\n"),
                    KeyCode::Backspace => editor.delete_at_cursor(1),

                    // ARROWS
                    KeyCode::Left => editor.move_cursor_left(),
                    KeyCode::Right => editor.move_cursor_right(),
                    KeyCode::Up => editor.move_cursor_up(),
                    KeyCode::Down => editor.move_cursor_down(),

                    // HOME / END
                    KeyCode::Home => editor.move_to_line_start(),
                    KeyCode::End => editor.move_to_line_end(),

                    // WORD MOVEMENT
                    KeyCode::Char('b') if key_event.modifiers == KeyModifiers::CONTROL => {
                        editor.move_word_left()
                    }
                    KeyCode::Char('f') if key_event.modifiers == KeyModifiers::CONTROL => {
                        editor.move_word_right()
                    }

                    _ => {}
                }
            }
        }
        render(editor, &mut stdout)?;
    }
}

fn render(editor: &Editor, stdout: &mut Stdout) -> std::io::Result<()> {
    execute!(stdout, Clear(ClearType::All))?;
    execute!(stdout, MoveTo(0, 0))?;

    let text = crate::rope::report(&editor.rope);
    let mut row = 0;
    let mut col = 0;

    for (i, ch) in text.chars().enumerate() {
        if ch == '\n' {
            row += 1;
            col = 0;
            writeln!(stdout)?;
            execute!(stdout, MoveTo(0, row as u16))?;
            continue;
        }

        if i == editor.cursor_index {
            write!(stdout, "\x1b[7m{}\x1b[0m", ch)?;
        } else {
            write!(stdout, "{}", ch)?;
        }

        col += 1;
    }

    if editor.cursor_index == text.len() {
        execute!(
            stdout,
            MoveTo(editor.cursor_col as u16, editor.cursor_row as u16)
        )?;
        write!(stdout, "\x1b[7m \x1b[0m")?;
    }

    execute!(
        stdout,
        MoveTo(editor.cursor_col as u16, editor.cursor_row as u16)
    )?;

    let (cols, rows) = terminal::size()?;
    execute!(stdout, MoveTo(0, rows - 1))?;

    let filename = editor.filename.clone().unwrap_or("[No Name]".into());
    let status = format!(
        "{} | Ln {}, Col {} | Ctrl+A Save | Ctrl+O Open | Ctrl+Q Quit",
        filename,
        editor.cursor_row + 1,
        editor.cursor_col + 1,
    );

    write!(
        stdout,
        "\x1b[7m{:<width$}\x1b[0m",
        status,
        width = cols as usize
    )?;

    stdout.flush()?;
    Ok(())
}

fn prompt(stdout: &mut Stdout, lable: &str) -> std::io::Result<(String)> {
    terminal::disable_raw_mode()?;
    execute!(stdout, Clear(ClearType::All), MoveTo(0, 0))?;
    write!(stdout, "{}: ", lable)?;
    stdout.flush()?;

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    terminal::enable_raw_mode()?;

    Ok(input.trim().to_string())
}

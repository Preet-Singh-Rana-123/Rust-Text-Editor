use crate::rope::{NodeRef, RopeNode, delete, insert, report};
use core::str;
use std::str::Chars;
use std::{char, fs};
use std::{
    io::{self, Write},
    usize,
};

pub struct Editor {
    pub rope: NodeRef,
    pub filename: Option<String>,
    pub cursor_index: usize,
    pub cursor_row: usize,
    pub cursor_col: usize,
    pub history: Vec<NodeRef>,
    pub future: Vec<NodeRef>,
}

impl Editor {
    pub fn new() -> Self {
        Editor {
            rope: RopeNode::new_leaf(""),
            filename: None,
            cursor_index: 0,
            cursor_row: 0,
            cursor_col: 0,
            history: Vec::new(),
            future: Vec::new(),
        }
    }

    // FILE OPERATIONS

    pub fn open_file(&mut self, filename: &str) -> io::Result<()> {
        let content = fs::read_to_string(filename)?;
        self.rope = RopeNode::new_leaf(&content);
        self.cursor_index = 0;
        self.cursor_row = 0;
        self.cursor_col = 0;
        self.update_cursor_position();
        self.filename = Some(filename.to_string());
        Ok(())
    }

    pub fn save(&self) -> io::Result<()> {
        if let Some(name) = &self.filename {
            let text = report(&self.rope);
            fs::write(name, text)?;
        }
        Ok(())
    }

    pub fn save_as(&mut self, name: &str) -> io::Result<()> {
        self.filename = Some(name.to_string());
        self.save()
    }

    // TEXT EDITING

    pub fn insert_at_cursor(&mut self, text: &str) {
        self.save_history();
        let processed = unescape(text);
        self.rope = insert(self.rope.clone(), self.cursor_index, &processed);
        self.cursor_index += processed.chars().count();
        self.update_cursor_position();
    }

    pub fn delete_at_cursor(&mut self, count: usize) {
        self.save_history();
        let end = self.cursor_index;
        let start = (self.cursor_index - count).min(self.length());
        self.rope = delete(self.rope.clone(), start, end);
        self.cursor_index = start.min(self.length());
        self.update_cursor_position();
    }

    // CURSOR MOVEMENT

    pub fn move_cursor_left(&mut self) {
        if self.cursor_index > 0 {
            self.cursor_index -= 1;
            self.update_cursor_position();
        }
    }

    pub fn move_cursor_right(&mut self) {
        let len = self.length();
        if self.cursor_index < len {
            self.cursor_index += 1;
            self.update_cursor_position();
        }
    }

    pub fn move_cursor_start(&mut self) {
        self.cursor_index = 0;
        self.update_cursor_position();
    }

    pub fn move_cursor_end(&mut self) {
        self.cursor_index = self.length();
        self.update_cursor_position();
    }

    pub fn move_cursor_up(&mut self) {
        if self.cursor_index == 0 {
            return;
        }

        let text = report(&self.rope);
        let lines: Vec<&str> = text.lines().collect();

        let target_row = self.cursor_row - 1;
        let target_col = self.cursor_col.min(lines[target_row].chars().count());

        let new_index: usize = lines
            .iter()
            .take(target_row)
            .map(|l| l.chars().count() + 1)
            .sum::<usize>()
            + target_col;

        self.cursor_index = new_index.min(self.length());
        self.update_cursor_position();
    }

    pub fn move_cursor_down(&mut self) {
        let text = report(&self.rope);
        let lines: Vec<&str> = text.lines().collect();
        if self.cursor_row + 1 >= lines.len() {
            return;
        }

        let target_row = self.cursor_row + 1;
        let target_col = self.cursor_col.min(lines[target_row].chars().count());

        let new_index: usize = lines
            .iter()
            .take(target_row)
            .map(|l| l.chars().count() + 1)
            .sum::<usize>()
            + target_col;

        self.cursor_index = new_index.min(self.length());
        self.update_cursor_position();
    }

    // START / END PER LINE

    pub fn move_to_line_end(&mut self) {
        let text = report(&self.rope);
        let lines: Vec<&str> = text.lines().collect();
        if self.cursor_row >= lines.len() {
            return;
        }
        let new_index: usize = lines
            .iter()
            .take(self.cursor_row + 1)
            .map(|l| l.chars().count() + 1)
            .sum::<usize>()
            - 1;
        self.cursor_index = new_index.min(self.length());
        self.update_cursor_position();
    }

    pub fn move_to_line_start(&mut self) {
        let text = report(&self.rope);
        let mut index = 0;
        let mut row = 0;
        for ch in text.chars() {
            if row == self.cursor_row {
                break;
            }
            index += 1;
            if ch == '\n' {
                row += 1;
            }
        }
        self.cursor_index = index;
        self.update_cursor_position();
    }

    // WORD LEVEL MOVEMENT

    pub fn move_word_right(&mut self) {
        let text = report(&self.rope);
        let chars: Vec<char> = text.chars().collect();
        let mut i = self.cursor_index;
        while i < chars.len() && chars[i].is_whitespace() {
            i += 1;
        }
        while i < chars.len() && !chars[i].is_whitespace() {
            i += 1;
        }
        self.cursor_index = i.min(chars.len());
        self.update_cursor_position();
    }

    pub fn move_word_left(&mut self) {
        if self.cursor_index == 0 {
            return;
        }
        let text = report(&self.rope);
        let chars: Vec<char> = text.chars().collect();
        let mut i = self.cursor_index.saturating_sub(1);
        while i > 0 && chars[i].is_whitespace() {
            i -= 1;
        }
        while i > 0 && !chars[i - 1].is_whitespace() {
            i -= 1;
        }
        self.cursor_index = i;
        self.update_cursor_position();
    }

    // LINE INSERTION AND DELETION

    pub fn insert_newline_above(&mut self) {
        self.save_history();
        self.move_to_line_start();
        self.insert_at_cursor("\n");
        self.cursor_index -= 1;
        self.update_cursor_position();
    }

    pub fn insert_newline_below(&mut self) {
        self.save_history();
        self.move_to_line_end();
        self.insert_at_cursor("\n");
        self.update_cursor_position();
    }

    pub fn delete_current_line(&mut self) {
        self.save_history();
        let text = report(&self.rope);
        let lines: Vec<&str> = text.lines().collect();
        if lines.is_empty() {
            return;
        }

        let start: usize = lines
            .iter()
            .take(self.cursor_row)
            .map(|l| l.chars().count() + 1)
            .sum::<usize>();

        let end: usize = start
            + lines[self.cursor_row].chars().count()
            + if self.cursor_row + 1 < lines.len() {
                1
            } else {
                0
            };
        self.rope = delete(self.rope.clone(), start, end);
        self.cursor_index = start.min(self.length());
        self.update_cursor_position();
    }

    // UNDO & REDO OPERATIONS

    pub fn undo(&mut self) {
        if let Some(prev) = self.history.pop() {
            self.future.push(self.rope.clone());
            self.rope = prev;
            self.cursor_index = self.cursor_index.min(self.length());
            self.update_cursor_position();
            println!("Undo Done");
        }
    }

    pub fn redo(&mut self) {
        if let Some(next) = self.future.pop() {
            self.history.push(self.rope.clone());
            self.rope = next;
            self.cursor_index = self.cursor_index.min(self.length());
            self.update_cursor_position();
            println!("Redo Done");
        }
    }

    // UTILITIES

    pub fn save_history(&mut self) {
        if self.history.len() > 1000 {
            self.history.remove(0);
        }
        self.history.push(self.rope.clone());
        self.future.clear();
    }

    pub fn update_cursor_position(&mut self) {
        let text = report(&self.rope);
        let mut row = 0;
        let mut col = 0;
        let mut chars_seen = 0;

        for ch in text.chars() {
            if chars_seen == self.cursor_index {
                break;
            }

            if ch == '\n' {
                row += 1;
                col = 0;
            } else {
                col += 1;
            }
            chars_seen += 1;
        }

        self.cursor_row = row;
        self.cursor_col = col;
    }

    pub fn display(&self) {
        let text = report(&self.rope);
        let mut displayed = String::new();
        for (i, ch) in text.chars().enumerate() {
            if i == self.cursor_index {
                displayed.push('|');
            }
            displayed.push(ch);
        }
        if self.cursor_index == text.chars().count() {
            displayed.push('|');
        }

        println!("\n--- TEXT BUFFER ---");
        println!("{}", displayed);
        println!("-------------------");
        println!(
            "Cursor at: line {}, column {}, index {}\n",
            self.cursor_row + 1,
            self.cursor_col + 1,
            self.cursor_index
        );
    }

    pub fn length(&self) -> usize {
        if let Some(node) = &self.rope {
            return node.borrow().length();
        }
        0
    }
}

fn unescape(text: &str) -> String {
    text.replace("\\n", "\n")
        .replace("\\t", "\t")
        .replace("\\r", "\r")
        .replace("\\\\", "\\")
}

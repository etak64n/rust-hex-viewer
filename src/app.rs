use std::fmt::Write as FmtWrite;

use ratatui::text::{Line, Text};

/// Represents a single rendered row and its starting offset.
#[derive(Clone)]
pub struct RowText {
    pub offset: usize,
    pub text: String,
    pub bytes: Vec<u8>,
}

/// Holds file data and exposes helpers for rendering and navigation.
pub struct App {
    file_name: String,
    bytes: Vec<u8>,
    scroll_row: usize,
    bytes_per_row: usize,
    view_rows: usize,
}

impl App {
    pub fn new(file_name: String, bytes: Vec<u8>, bytes_per_row: usize) -> Self {
        Self {
            file_name,
            bytes,
            scroll_row: 0,
            bytes_per_row: bytes_per_row.max(1),
            view_rows: 1,
        }
    }

    pub fn file_name(&self) -> &str {
        &self.file_name
    }

    pub fn set_view_rows(&mut self, rows: usize) {
        self.view_rows = rows.max(1);
    }

    pub fn view_rows(&self) -> usize {
        self.view_rows
    }

    pub fn scroll_rows(&mut self, delta: isize) {
        if self.total_rows() == 0 {
            self.scroll_row = 0;
            return;
        }
        let max_row = self.total_rows() - 1;
        let next = (self.scroll_row as isize + delta).clamp(0, max_row as isize);
        self.scroll_row = next as usize;
    }

    pub fn scroll_to_start(&mut self) {
        self.scroll_row = 0;
    }

    pub fn scroll_to_end(&mut self) {
        if self.total_rows() == 0 {
            self.scroll_row = 0;
        } else {
            self.scroll_row = self.total_rows() - 1;
        }
    }

    pub fn render_lines(&self, rows: usize) -> Text<'static> {
        if self.bytes.is_empty() {
            return Text::from(vec![Line::from("File is empty.")]);
        }

        let mut rows_with_text = self.lines_for_range(self.scroll_row, rows);
        if rows_with_text.is_empty() {
            rows_with_text.push(RowText {
                offset: 0,
                text: "(End of file)".to_string(),
                bytes: Vec::new(),
            });
        }

        let tui_lines: Vec<Line> = rows_with_text
            .into_iter()
            .map(|row| Line::from(row.text))
            .collect();
        Text::from(tui_lines)
    }

    pub fn lines_for_range(&self, start_row: usize, rows: usize) -> Vec<RowText> {
        if self.bytes.is_empty() || rows == 0 {
            return Vec::new();
        }

        let total_rows = self.total_rows();
        let limit = start_row.saturating_add(rows).min(total_rows);
        let mut lines = Vec::with_capacity(limit.saturating_sub(start_row));

        for row in start_row..limit {
            let offset = row * self.bytes_per_row;
            let end = (offset + self.bytes_per_row).min(self.bytes.len());
            let chunk = &self.bytes[offset..end];
            lines.push(RowText {
                offset,
                text: format_line(offset, chunk, self.bytes_per_row),
                bytes: chunk.to_vec(),
            });
        }

        lines
    }

    pub fn status_line(&self) -> String {
        let total_rows = self.total_rows().max(1);
        format!(
            "{} | bytes: {} | row: {}/{} | offset: 0x{:08X} | press q to quit",
            self.file_name,
            self.bytes.len(),
            self.scroll_row.saturating_add(1).min(total_rows),
            total_rows,
            self.current_offset()
        )
    }

    pub fn total_rows(&self) -> usize {
        if self.bytes.is_empty() {
            0
        } else {
            (self.bytes.len() + self.bytes_per_row - 1) / self.bytes_per_row
        }
    }

    pub fn bytes_len(&self) -> usize {
        self.bytes.len()
    }

    pub fn bytes_per_row(&self) -> usize {
        self.bytes_per_row
    }

    fn current_offset(&self) -> usize {
        self.scroll_row * self.bytes_per_row
    }
}

fn format_line(offset: usize, chunk: &[u8], width: usize) -> String {
    let mut hex_buf = String::with_capacity(width * 3 + 8);
    for idx in 0..width {
        if idx == width / 2 {
            hex_buf.push(' ');
        }
        if let Some(&byte) = chunk.get(idx) {
            let _ = write!(hex_buf, "{:02X}", byte);
        } else {
            hex_buf.push_str("  ");
        }
        if idx + 1 != width {
            hex_buf.push(' ');
        }
    }

    let mut ascii_buf = String::with_capacity(width);
    for idx in 0..width {
        if let Some(&byte) = chunk.get(idx) {
            ascii_buf.push(printable(byte));
        } else {
            ascii_buf.push(' ');
        }
    }

    format!("{offset:08X}  {hex_buf}  |{ascii_buf}|")
}

fn printable(byte: u8) -> char {
    if byte.is_ascii_graphic() || byte == b' ' {
        byte as char
    } else {
        '.'
    }
}

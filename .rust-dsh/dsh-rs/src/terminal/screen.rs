//! A terminal screen that serialises exactly as xterm's `SerializeAddon` does.
//!
//! The TypeScript host keeps each terminal in `@xterm/headless` and sends a new
//! attachment `SerializeAddon.serialize()` as its `screen`. A client replays that
//! text into its own terminal, so the text has to be the same one xterm would
//! make, byte for byte. This module holds a `vt100` parser (vendored in
//! `vendor/vt100` with the extra attributes xterm tracks) and ports
//! `StringSerializeHandler` from `@xterm/addon-serialize` 0.14.

use vt100::{Attrs, Buffer, Color, MouseProtocolMode, Parser, Row};

/// A headless terminal and its serialiser.
pub struct Screen {
    parser: Parser,
}

impl Screen {
    /// A screen of `cols` by `rows` keeping `scrollback` lines.
    pub fn new(cols: u16, rows: u16, scrollback: usize) -> Self {
        Screen { parser: Parser::new(rows, cols, scrollback) }
    }

    /// Feed terminal output.
    pub fn write(&mut self, data: &[u8]) {
        self.parser.process(data);
    }

    /// Change the size.
    pub fn resize(&mut self, cols: u16, rows: u16) {
        self.parser.set_size(rows, cols);
    }

    /// `SerializeAddon.serialize()` with its default options.
    pub fn serialize(&self) -> String {
        let screen = self.parser.screen();
        let (rows, cols) = screen.size();
        let mut content = Handler::new(screen.buffer(false), rows, cols, screen.current_attrs()).serialize();
        if screen.alternate_buffer_active() {
            let alternate = Handler::new(screen.buffer(true), rows, cols, screen.current_attrs()).serialize();
            content.push_str("\u{1b}[?1049h\u{1b}[H");
            content.push_str(&alternate);
        }
        content.push_str(&modes(screen));
        content
    }
}

fn modes(screen: &vt100::Screen) -> String {
    let mut out = String::new();
    if screen.application_cursor() {
        out.push_str("\u{1b}[?1h");
    }
    if screen.application_keypad() {
        out.push_str("\u{1b}[?66h");
    }
    if screen.bracketed_paste() {
        out.push_str("\u{1b}[?2004h");
    }
    match screen.mouse_protocol_mode() {
        MouseProtocolMode::None => {}
        MouseProtocolMode::Press => out.push_str("\u{1b}[?9h"),
        MouseProtocolMode::PressRelease => out.push_str("\u{1b}[?1000h"),
        MouseProtocolMode::ButtonMotion => out.push_str("\u{1b}[?1002h"),
        MouseProtocolMode::AnyMotion => out.push_str("\u{1b}[?1003h"),
    }
    out
}

// --- the string serialiser ---------------------------------------------------

fn is_default(a: &Attrs) -> bool {
    *a == Attrs::default()
}

fn flags_equal(a: &Attrs, b: &Attrs) -> bool {
    a.inverse() == b.inverse()
        && a.bold() == b.bold()
        && a.underline() == b.underline()
        && a.overline == b.overline
        && a.blink() == b.blink()
        && a.invisible() == b.invisible()
        && a.italic() == b.italic()
        && a.dim() == b.dim()
        && a.strike() == b.strike()
}

/// The SGR parameters that turn the style `old` into `cell`.
fn diff_style(cell: &Attrs, old: &Attrs) -> Vec<u32> {
    let mut seq = Vec::new();
    let fg_changed = cell.fgcolor != old.fgcolor;
    let bg_changed = cell.bgcolor != old.bgcolor;
    let flags_changed = !flags_equal(cell, old);
    if !(fg_changed || bg_changed || flags_changed) {
        return seq;
    }
    if is_default(cell) {
        if !is_default(old) {
            seq.push(0);
        }
        return seq;
    }
    let color = |seq: &mut Vec<u32>, color: Color, extended: u32, base: u32, bright: u32, default: u32| match color {
        Color::Rgb(r, g, b) => seq.extend([extended, 2, u32::from(r), u32::from(g), u32::from(b)]),
        Color::Idx(n) if n >= 16 => seq.extend([extended, 5, u32::from(n)]),
        Color::Idx(n) => seq.push(if n & 8 != 0 { bright + u32::from(n & 7) } else { base + u32::from(n & 7) }),
        Color::Default => seq.push(default),
    };
    if fg_changed {
        color(&mut seq, cell.fgcolor, 38, 30, 90, 39);
    }
    if bg_changed {
        color(&mut seq, cell.bgcolor, 48, 40, 100, 49);
    }
    if flags_changed {
        let toggles: [(bool, bool, u32, u32); 9] = [
            (cell.inverse(), old.inverse(), 7, 27),
            (cell.bold(), old.bold(), 1, 22),
            (cell.underline(), old.underline(), 4, 24),
            (cell.overline, old.overline, 53, 55),
            (cell.blink(), old.blink(), 5, 25),
            (cell.invisible(), old.invisible(), 8, 28),
            (cell.italic(), old.italic(), 3, 23),
            (cell.dim(), old.dim(), 2, 22),
            (cell.strike(), old.strike(), 9, 29),
        ];
        for (now, before, on, off) in toggles {
            if now != before {
                seq.push(if now { on } else { off });
            }
        }
    }
    seq
}

/// One cell as the serialiser sees it.
struct Cell<'a> {
    chars: String,
    width: usize,
    attrs: &'a Attrs,
}

fn cell_at<'a>(row: &'a Row, col: u16) -> Option<Cell<'a>> {
    let cell = row.get(col)?;
    let width = if cell.is_wide_continuation() {
        0
    } else if cell.is_wide() {
        2
    } else {
        1
    };
    Some(Cell { chars: cell.contents(), width, attrs: cell.attrs() })
}

struct Handler<'a> {
    rows: Vec<&'a Row>,
    cursor_row: usize,
    cursor_col: usize,
    terminal_rows: usize,
    cols: u16,
    current_attrs: Attrs,

    all_rows: Vec<String>,
    separators: Vec<String>,
    current_row: String,
    null_cells: usize,
    cursor_style: Attrs,
    cursor_style_row: usize,
    cursor_style_col: u16,
    background: Attrs,
    first_row: usize,
    last_cursor_row: usize,
    last_cursor_col: usize,
    last_content_row: usize,
    last_content_col: usize,
}

impl<'a> Handler<'a> {
    fn new(buffer: Buffer<'a>, terminal_rows: u16, cols: u16, current_attrs: Attrs) -> Self {
        Handler {
            rows: buffer.rows,
            cursor_row: buffer.cursor_row,
            cursor_col: buffer.cursor_col,
            terminal_rows: usize::from(terminal_rows),
            cols,
            current_attrs,
            all_rows: Vec::new(),
            separators: Vec::new(),
            current_row: String::new(),
            null_cells: 0,
            cursor_style: Attrs::default(),
            cursor_style_row: 0,
            cursor_style_col: 0,
            background: Attrs::default(),
            first_row: 0,
            last_cursor_row: 0,
            last_cursor_col: 0,
            last_content_row: 0,
            last_content_col: 0,
        }
    }

    fn serialize(mut self) -> String {
        let end = self.rows.len();
        self.first_row = 0;
        for row in 0..end {
            let line = self.rows[row];
            for col in 0..self.cols {
                if let Some(cell) = cell_at(line, col) {
                    self.next_cell(&cell, row, col);
                }
            }
            self.row_end(row, row + 1 == end);
        }
        self.finish()
    }

    fn next_cell(&mut self, cell: &Cell<'_>, row: usize, col: u16) {
        if cell.width == 0 {
            return;
        }
        let empty = cell.chars.is_empty();
        let sgr = diff_style(cell.attrs, &self.cursor_style);
        let style_changed = if empty { self.cursor_style.bgcolor != cell.attrs.bgcolor } else { !sgr.is_empty() };
        if style_changed {
            if self.null_cells > 0 {
                if self.cursor_style.bgcolor != self.background.bgcolor {
                    self.current_row.push_str(&format!("\u{1b}[{}X", self.null_cells));
                }
                self.current_row.push_str(&format!("\u{1b}[{}C", self.null_cells));
                self.null_cells = 0;
            }
            self.last_content_row = row;
            self.last_cursor_row = row;
            self.last_content_col = usize::from(col);
            self.last_cursor_col = usize::from(col);
            let joined: Vec<String> = sgr.iter().map(u32::to_string).collect();
            self.current_row.push_str(&format!("\u{1b}[{}m", joined.join(";")));
            self.cursor_style = *cell.attrs;
            self.cursor_style_row = row;
            self.cursor_style_col = col;
        }
        if empty {
            self.null_cells += cell.width;
        } else {
            if self.null_cells > 0 {
                if self.cursor_style.bgcolor == self.background.bgcolor {
                    self.current_row.push_str(&format!("\u{1b}[{}C", self.null_cells));
                } else {
                    self.current_row.push_str(&format!("\u{1b}[{}X", self.null_cells));
                    self.current_row.push_str(&format!("\u{1b}[{}C", self.null_cells));
                }
                self.null_cells = 0;
            }
            self.current_row.push_str(&cell.chars);
            self.last_content_row = row;
            self.last_cursor_row = row;
            self.last_content_col = usize::from(col) + cell.width;
            self.last_cursor_col = usize::from(col) + cell.width;
        }
    }

    fn row_end(&mut self, row: usize, is_last: bool) {
        if self.null_cells > 0 && self.cursor_style.bgcolor != self.background.bgcolor {
            self.current_row.push_str(&format!("\u{1b}[{}X", self.null_cells));
        }
        let mut separator = String::new();
        if !is_last {
            if row - self.first_row >= self.terminal_rows {
                if let Some(cell) = self.rows.get(self.cursor_style_row).and_then(|line| cell_at(line, self.cursor_style_col)) {
                    self.background = *cell.attrs;
                }
            }
            let current = self.rows[row];
            let next_wrapped = current.wrapped();
            if !next_wrapped {
                separator.push_str("\r\n");
                self.last_cursor_row = row + 1;
                self.last_cursor_col = 0;
            } else {
                let length = self.cols;
                let next = self.rows[row + 1];
                let last = cell_at(current, length - 1);
                let second_last = if length >= 2 { cell_at(current, length - 2) } else { None };
                let next_first = cell_at(next, 0);
                let (last, next_first) = match (last, next_first) {
                    (Some(l), Some(n)) => (l, n),
                    _ => return self.push_row(separator),
                };
                let next_double = next_first.width > 1;
                let mut valid = false;
                // JavaScript precedence: `a && b ? c : d` is `(a && b) ? c : d`.
                let gate = if !next_first.chars.is_empty() && next_double { self.null_cells <= 1 } else { self.null_cells == 0 };
                if gate {
                    if (!last.chars.is_empty() || last.width == 0) && last.attrs.bgcolor == next_first.attrs.bgcolor {
                        valid = true;
                    }
                    if next_double {
                        if let Some(second) = &second_last {
                            if (!second.chars.is_empty() || second.width == 0)
                                && last.attrs.bgcolor == next_first.attrs.bgcolor
                                && second.attrs.bgcolor == next_first.attrs.bgcolor
                            {
                                valid = true;
                            }
                        }
                    }
                }
                if !valid {
                    separator.push_str(&"-".repeat(self.null_cells + 1));
                    separator.push_str("\u{1b}[1D\u{1b}[1X");
                    if self.null_cells > 0 {
                        separator.push_str("\u{1b}[A");
                        separator.push_str(&format!("\u{1b}[{}C", usize::from(length) - self.null_cells));
                        separator.push_str(&format!("\u{1b}[{}X", self.null_cells));
                        separator.push_str(&format!("\u{1b}[{}D", usize::from(length) - self.null_cells));
                        separator.push_str("\u{1b}[B");
                    }
                    self.last_content_row = row + 1;
                    self.last_content_col = 0;
                    self.last_cursor_row = row + 1;
                    self.last_cursor_col = 0;
                }
            }
        }
        self.push_row(separator);
    }

    fn push_row(&mut self, separator: String) {
        self.all_rows.push(std::mem::take(&mut self.current_row));
        self.separators.push(separator);
        self.null_cells = 0;
    }

    fn finish(mut self) -> String {
        let mut row_end = self.all_rows.len();
        if self.rows.len() - self.first_row <= self.terminal_rows {
            row_end = self.last_content_row + 1 - self.first_row;
            self.last_cursor_col = self.last_content_col;
            self.last_cursor_row = self.last_content_row;
        }
        let mut content = String::new();
        for i in 0..row_end {
            content.push_str(&self.all_rows[i]);
            if i + 1 < row_end {
                content.push_str(&self.separators[i]);
            }
        }
        let moved = self.cursor_row != self.last_cursor_row || self.cursor_col != self.last_cursor_col;
        if moved {
            let down = self.cursor_row as i64 - self.last_cursor_row as i64;
            if down > 0 {
                content.push_str(&format!("\u{1b}[{down}B"));
            } else if down < 0 {
                content.push_str(&format!("\u{1b}[{}A", -down));
            }
            let right = self.cursor_col as i64 - self.last_cursor_col as i64;
            if right > 0 {
                content.push_str(&format!("\u{1b}[{right}C"));
            } else if right < 0 {
                content.push_str(&format!("\u{1b}[{}D", -right));
            }
        }
        let sgr = diff_style(&self.current_attrs, &self.cursor_style);
        if !sgr.is_empty() {
            let joined: Vec<String> = sgr.iter().map(u32::to_string).collect();
            content.push_str(&format!("\u{1b}[{}m", joined.join(";")));
        }
        content
    }
}

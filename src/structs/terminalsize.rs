use rustix_openpty::rustix::termios::Winsize;

#[derive(Clone, Copy)]
pub struct TerminalSize {
    pub cols: usize,
    pub rows: usize,
}

impl TerminalSize {
    pub fn new(cols: usize, rows: usize) -> Self {
        Self { cols, rows }
    }

    pub fn winsize(&self) -> Winsize {
        Winsize {
            ws_col: u16::try_from(self.cols).expect("Terminal is too wide for Winsize"),
            ws_row: u16::try_from(self.rows).expect("Terminal is too tall for Winsize"),
            ws_xpixel: 0,
            ws_ypixel: 0,
        }
    }
}

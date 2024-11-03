#[derive(Debug, Clone)]
pub enum TerminalOutput {
    Text(String),
    NewLine,
    CarriageReturn,
    Backspace,
}

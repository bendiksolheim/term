use super::*;

use std::fmt::Write;

#[test]
fn test_cursor_pos() {
    let pos = AnsiSequence::CursorPos(5, 20);
    let mut buff = String::new();

    write!(&mut buff, "{}", pos).expect("failed to write");

    assert_eq!(buff, "\x1b[5;20H");
}

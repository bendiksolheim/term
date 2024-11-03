use iced::futures::{
    channel::mpsc::{self, Sender},
    SinkExt, Stream, StreamExt,
};
use libc::{winsize, TIOCSCTTY, TIOCSWINSZ};
pub use rustix_openpty::rustix::termios::Winsize;
use std::{
    fs::File,
    io::{self, Error, Write},
    os::{
        fd::{AsRawFd, OwnedFd},
        unix::process::CommandExt,
    },
    process::{Child, Command},
    time::Duration,
};

use super::{
    pty_reader::{PtyReader, PtyReaderResult},
    terminal_output::TerminalOutput,
};

pub struct Term {}

impl Term {
    pub fn spawn(winsize: Winsize) -> impl Stream<Item = Event> {
        let pty = open_pty(winsize).expect("Could not get PTY");
        let master = File::from(pty.master.try_clone().unwrap());

        iced::stream::channel(100, |mut output| async move {
            let shell = std::env::var("SHELL").expect("$SHELL is not set");
            read_output(&pty.master, output.clone());
            let _ = spawn_shell(&pty.slave, shell.as_str());
            let (sender, mut receiver) = mpsc::channel(100);
            output
                .send(Event::Ready(sender))
                .await
                .expect("Could not send Message::Ready");

            loop {
                let input = receiver.select_next_some().await;
                match input {
                    TermMessage::Bytes(bytes) => {
                        write_bytes(&master, &bytes);
                    }

                    TermMessage::WindowResized(columns, rows) => match resize(&master, columns, rows) {
                        Ok(_) => {
                            println!("Resize successful");
                        }
                        Err(err) => {
                            println!("Error resizing: {:?}", err);
                        }
                    },
                }
            }
        })
    }
}

fn write_bytes(mut master: &File, content: &[u8]) {
    master.write_all(content).unwrap();
    master.flush().unwrap();
}

fn resize(master: &File, columns: usize, rows: usize) -> Result<(), std::io::Error> {
    let size = winsize {
        ws_row: rows as u16,
        ws_col: columns as u16,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };
    println!("New size in term: {:?}", size);

    let result = unsafe { libc::ioctl(master.as_raw_fd(), TIOCSWINSZ, &size) };

    if result == 0 {
        Ok(())
    } else {
        Err(std::io::Error::last_os_error())
    }
}

#[derive(Debug, Clone)]
pub enum Event {
    Ready(mpsc::Sender<TermMessage>),
    Output(Vec<TerminalOutput>),
}

type Columns = usize;
type Rows = usize;

#[derive(Debug, Clone)]
pub enum TermMessage {
    Bytes(Vec<u8>),
    WindowResized(Columns, Rows),
}

struct Pty {
    master: OwnedFd,
    slave: OwnedFd,
}

fn open_pty(winsize: Winsize) -> Result<Pty, Error> {
    // Ask OS for a PTY
    let pty = rustix_openpty::openpty(None, Some(&winsize))?;

    // Make reads on master non-blockinginput
    unsafe { libc::fcntl(pty.controller.as_raw_fd(), libc::F_SETFL, libc::O_NONBLOCK) };

    // Return a struct with master and slave side of PTY
    Ok(Pty {
        master: pty.controller,
        slave: pty.user,
    })
}

fn read_output(master: &OwnedFd, mut sender: Sender<Event>) {
    let master_clone = master.try_clone().unwrap();

    let _a = async_std::task::spawn(async move {
        let file = File::from(master_clone);
        let mut reader = PtyReader::new(file);

        loop {
            async_std::task::sleep(Duration::from_millis(16)).await;
            match reader.read_chunk() {
                PtyReaderResult::MoreLeft => {}
                PtyReaderResult::EndOfInput => match reader.process_buffer() {
                    Some(output) => {
                        sender
                            .send(Event::Output(output))
                            .await
                            .expect("Could not send output to terminal GUI");
                    }
                    None => {}
                },
            }
        }
    });
}
fn spawn_shell(slave: &OwnedFd, shell: &str) -> io::Result<Child> {
    let mut command = Command::new(shell);
    command.env("TERM", "xterm");
    command.stdin(slave.try_clone()?);
    command.stdout(slave.try_clone()?);
    command.stderr(slave.try_clone()?);

    let slave_fd = slave.as_raw_fd();
    unsafe {
        command.pre_exec(move || {
            // Become leader of new session
            let err = libc::setsid();
            if err == -1 {
                return Err(Error::new(io::ErrorKind::Other, "Failed to set session ID"));
            }

            // Set controlling terminal
            let res = libc::ioctl(slave_fd, TIOCSCTTY as _, 0);
            if res < 0 {
                return Err(Error::new(io::ErrorKind::Other, "Failed to set controlling terminal"));
            }

            libc::close(slave_fd);

            Ok(())
        });
    }

    command.spawn()
}

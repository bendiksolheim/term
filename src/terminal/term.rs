use iced::{
    futures::{
        channel::mpsc::{self, Sender},
        SinkExt, Stream, StreamExt,
    },
    keyboard::Key,
};
use libc::TIOCSCTTY;
pub use rustix_openpty::rustix::termios::Winsize;
use std::{
    fs::File,
    io::{self, Error, ErrorKind, Read, Write},
    os::{
        fd::{AsRawFd, OwnedFd},
        unix::process::CommandExt,
    },
    process::{Child, Command},
    time::Duration,
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
                    TermMessage::Input(input) => match input {
                        Key::Named(named) => match named {
                            iced::keyboard::key::Named::Enter => send(&master, "\r"),
                            iced::keyboard::key::Named::Space => send(&master, " "),
                            iced::keyboard::key::Named::Backspace => send(&master, "\x7f"),
                            _ => {}
                        },
                        Key::Character(c) => {
                            send(&master, c.as_str());
                        }
                        Key::Unidentified => todo!(),
                    },
                }
            }
        })
    }
}

fn send(mut master: &File, content: &str) {
    master.write_all(content.as_bytes()).unwrap();
    master.flush().unwrap();
}

#[derive(Debug, Clone)]
pub enum Event {
    Ready(mpsc::Sender<TermMessage>),
    Output(Vec<Output>),
}

#[derive(Debug, Clone)]
pub enum Output {
    Text(String),
    NewLine,
    CarriageReturn,
    Backspace,
}

#[derive(Debug, Clone)]
pub enum TermMessage {
    Input(Key),
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
        std::io::stdout().flush().unwrap();
        let mut buffer = [0u8; 1024];
        let mut file = File::from(master_clone);
        let mut accumulated: Vec<u8> = vec![];
        loop {
            async_std::task::sleep(Duration::from_millis(16)).await;
            match file.read(&mut buffer) {
                Ok(0) => break,
                Ok(num_bytes) => {
                    let mut read_bytes = buffer[..num_bytes].to_vec();
                    accumulated.append(&mut read_bytes);
                }

                Err(e) => {
                    // WouldBlock is expected when there is no input
                    if e.kind() != ErrorKind::WouldBlock {
                        eprint!("Error reading from PTY: {:?}", e)
                    } else {
                        if accumulated.len() > 0 {
                            let mut byte_sequence: Vec<u8> = vec![];
                            let mut output: Vec<Output> = vec![];
                            for byte in accumulated.iter() {
                                match byte {
                                    b'\x08' => {
                                        if byte_sequence.len() > 0 {
                                            let _s = byte_sequence.drain(0..).collect();
                                            let str_sequence = String::from_utf8(_s).unwrap();
                                            output.push(Output::Text(str_sequence));
                                        }
                                        output.push(Output::Backspace);
                                    }
                                    b'\n' => {
                                        if byte_sequence.len() > 0 {
                                            let _s = byte_sequence.drain(0..).collect();
                                            let str_sequence = String::from_utf8(_s).unwrap();
                                            output.push(Output::Text(str_sequence));
                                        }
                                        output.push(Output::NewLine);
                                    }
                                    b'\r' => {
                                        if byte_sequence.len() > 0 {
                                            let _s = byte_sequence.drain(0..).collect();
                                            let str_sequence = String::from_utf8(_s).unwrap();
                                            output.push(Output::Text(str_sequence));
                                        }
                                        output.push(Output::CarriageReturn);
                                    }
                                    b => {
                                        byte_sequence.push(*b);
                                    }
                                }
                            }
                            let _s = String::from_utf8(byte_sequence.clone()).unwrap();
                            output.push(Output::Text(_s));
                            sender.send(Event::Output(output.clone())).await.unwrap();
                            accumulated.clear();
                        }
                    }
                }
            }
        }
    });
}

struct PtyReader<R: Read> {
    inner: R,
    buffer: Vec<u8>,
}

impl<R: Read> PtyReader<R> {
    fn new(inner: R) -> Self {
        Self {
            inner,
            buffer: Vec::new(),
        }
    }

    fn read_chunk(&mut self) -> std::io::Result<()> {
        let mut chunk = [0u8; 1024];
        let n = self.inner.read(&mut chunk)?;
        self.buffer.extend_from_slice(&chunk[..n]);
        Ok(())
    }

    fn process_buffer(&mut self) {
        self.buffer.drain(..);
    }
}

fn spawn_shell(slave: &OwnedFd, shell: &str) -> io::Result<Child> {
    let mut command = Command::new(shell);
    command.env("TERM", "xterm");
    // command.env("TERM", "dumb");
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

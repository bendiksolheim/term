use gpui::*;
use libc::TIOCSCTTY;
use std::ffi::c_int;
use std::fs::File;
use std::io::{self, Error, Read, Write};
use std::os::fd::{AsRawFd, OwnedFd};
use std::os::unix::process::CommandExt;
use std::process::{Child, Command};
use std::sync::{Arc, Mutex};

struct TerminalView {
    master: File,
    content: Arc<Mutex<Vec<String>>>,
    focus_handle: FocusHandle,
}

struct Pty {
    master: OwnedFd,
    slave: OwnedFd,
}

fn open_pty() -> Result<Pty, Error> {
    let pty = rustix_openpty::openpty(None, None)?;
    Ok(Pty {
        master: pty.controller,
        slave: pty.user,
    })
}

fn spawn_shell(slave: &OwnedFd, shell: &str) -> io::Result<Child> {
    let mut command = Command::new(shell);
    command.env("TERM", "xterm-256color");
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

fn read_output(master: &OwnedFd) -> Arc<Mutex<Vec<String>>> {
    let content = Arc::new(Mutex::new(vec![String::new()]));
    let content_clone = Arc::clone(&content);
    let master_clone = master.try_clone().unwrap();
    std::thread::spawn(move || {
        let mut buffer = [0u8; 1024];
        let mut file = File::from(master_clone);
        loop {
            match file.read(&mut buffer) {
                Ok(0) => break,
                Ok(num_bytes) => {
                    let mut _content = content_clone.lock().unwrap();
                    let bytes = &buffer[..num_bytes];
                    let plain_bytes = strip_ansi_escapes::strip(bytes);
                    println!("{}", String::from_utf8_lossy(bytes));
                    for byte in plain_bytes {
                        if byte == b'\n' {
                            _content.push(String::new());
                        } else {
                            let last_line = _content.last_mut().unwrap();
                            last_line.push(byte as char);
                        }
                    }
                }
                Err(e) => {
                    eprint!("Error reading from PTY: {:?}", e)
                }
            }
        }
    });

    content
}

fn lol(f: Tmp) {
    std::thread::spawn(move || {
        let mut _f = f;
        _f.f();
    });
}

pub struct Tmp {
    pub f: Arc<dyn Fn() + Sync + Send>,
}

impl Tmp {
    pub fn new(f: Arc<dyn Fn() + Send + Sync>) -> Self {
        Self { f }
    }
}

impl TerminalView {
    fn new(cx: &mut ViewContext<Self>) -> Result<Self, Error> {
        let shell = std::env::var("SHELL").expect("Expected to find default shell in $SHELL env var");
        let pty = open_pty()?;
        let content = read_output(&pty.master);
        cx.observe(Model<&content>, |_this, _model, cx| {
            cx.notify();
        });
        spawn_shell(&pty.slave, shell.as_str())
            .map(|_| Self {
                master: File::from(pty.master.try_clone().unwrap()),
                content,
                focus_handle: cx.focus_handle(),
            })
            .map_err(|err| Error::new(err.kind(), format!("Failed to spawn command '{}': {}", shell, err)))
    }

    fn send_input(&self, input: &str) {
        println!("Input: {}", input);
        let mut f = self.master.try_clone().unwrap();
        let _ = f.write_all(input.as_bytes());
        let _ = f.flush();
    }
}

impl Render for TerminalView {
    fn render(&mut self, cx: &mut ViewContext<TerminalView>) -> impl IntoElement {
        cx.focus(&self.focus_handle);

        let content = self.content.lock().unwrap();
        let content_clone = content.clone();
        drop(content);
        div()
            .pt_5()
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, cx| {
                match event.keystroke.key.as_str() {
                    "enter" => this.send_input("\r"),
                    "backspace" => this.send_input("\x7f"),
                    "space" => this.send_input(" "),
                    key if key.len() == 1 => this.send_input(key),
                    _ => {}
                }

                // Notify gpui of changes to the model
                cx.notify()
            }))
            .size_full()
            .bg(rgb(0x282c34))
            .text_color(rgb(0xabb2bf))
            .font(font("monospace"))
            .child(
                div().size_full().p_2().items_start().child(
                    div()
                        .flex_col()
                        .children(content_clone.iter().map(|line| div().child(format!("{}", line)))),
                ),
            )
    }
}

actions!(term, [Quit]);

fn main() {
    let window_options = WindowOptions {
        titlebar: Some(TitlebarOptions {
            appears_transparent: true,
            ..Default::default()
        }),
        ..Default::default()
    };
    App::new().run(|cx: &mut AppContext| {
        cx.activate(true);
        cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]);
        cx.on_action(|_: &Quit, cx| cx.quit());

        cx.open_window(window_options, |cx| cx.new_view(|cx| TerminalView::new(cx).unwrap()))
            .unwrap();
    });
}

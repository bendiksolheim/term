use gpui::*;
use libc::TIOCSCTTY;
use std::fs::File;
use std::io::{self, Error, ErrorKind, Read, Write};
use std::os::fd::{AsRawFd, OwnedFd};
use std::os::unix::process::CommandExt;
use std::process::{Child, Command};
use std::sync::{Arc, Mutex};
use std::time::Duration;

struct Content {
    content: Arc<Mutex<Vec<Vec<u8>>>>,
}

impl Content {
    fn new() -> Self {
        Self {
            content: Arc::new(Mutex::new(vec![vec![]])),
        }
    }
}

struct TerminalView {
    master: File,
    output: Model<Content>,
    strip_ansi: bool,
    focus_handle: FocusHandle,
}

struct Pty {
    master: OwnedFd,
    slave: OwnedFd,
}

fn open_pty() -> Result<Pty, Error> {
    // Ask OS for a PTY
    let pty = rustix_openpty::openpty(None, None)?;

    // Make reads on master non-blocking
    unsafe { libc::fcntl(pty.controller.as_raw_fd(), libc::F_SETFL, libc::O_NONBLOCK) };

    // Return a struct with master and slave side of PTY
    Ok(Pty {
        master: pty.controller,
        slave: pty.user,
    })
}

fn spawn_shell(slave: &OwnedFd, shell: &str) -> io::Result<Child> {
    let mut command = Command::new(shell);
    command.env("TERM", "dumb");
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

fn read_output(master: &OwnedFd, cx: &mut ViewContext<TerminalView>) -> Model<Content> {
    let content_model = cx.new_model(|_| Content::new());
    let content = content_model.clone();
    let master_clone = master.try_clone().unwrap();

    cx.spawn(|_, mut cx| async move {
        let mut buffer = [0u8; 1024];
        let mut file = File::from(master_clone);
        loop {
            async_std::task::sleep(Duration::from_millis(16)).await;
            match file.read(&mut buffer) {
                Ok(0) => break,
                Ok(num_bytes) => {
                    cx.update_model(&content, |model, cx| {
                        let mut _content = model.content.lock().unwrap();
                        let bytes = buffer[..num_bytes].to_vec();
                        if bytes == b"\n" {
                            // Newline
                            _content.push(vec![]);
                        } else if bytes == b"\x08 \x08" {
                            // Backspace
                            if let Some(line) = _content.last_mut() {
                                if line.len() > 0 {
                                    line.remove(line.len() - 1);
                                }
                            }
                        } else {
                            // All the rest
                            if let Some(line) = _content.last_mut() {
                                line.extend(bytes);
                            }
                        }
                        drop(_content);
                        cx.notify();
                    })
                    .unwrap();
                }
                Err(e) => {
                    // WouldBlock is expected when there is no input
                    if e.kind() != ErrorKind::WouldBlock {
                        eprint!("Error reading from PTY: {:?}", e)
                    }
                }
            }
        }
    })
    .detach();

    content_model
}

impl TerminalView {
    fn new(cx: &mut ViewContext<Self>) -> Result<Self, Error> {
        // Retrieve shell from env var
        let shell = std::env::var("SHELL").expect("Expected to find default shell in $SHELL env var");

        // Ask OS for a PTY
        let pty = open_pty()?;

        // Listen for input to master
        let output = read_output(&pty.master.try_clone().unwrap(), cx);

        // Observe changes in the model and notify gpui
        cx.observe(&output, |_, _, cx| {
            cx.notify();
        })
        .detach();

        // Spawn shell and connect to slave side of PTY
        let result = spawn_shell(&pty.slave, shell.as_str());

        // Return model if shell spawns without error
        result
            .map(|_| Self {
                master: File::from(pty.master.try_clone().unwrap()),
                output,
                strip_ansi: true,
                focus_handle: cx.focus_handle(),
            })
            .map_err(|err| Error::new(err.kind(), format!("Failed to spawn command '{}': {}", shell, err)))
    }

    fn send_input(&self, input: &str) {
        let mut f = self.master.try_clone().unwrap();
        let _ = f.write_all(input.as_bytes());
        let _ = f.flush();
    }

    fn toggle_ansi(&mut self) {
        self.strip_ansi = !self.strip_ansi;
    }
}

impl Render for TerminalView {
    fn render(&mut self, cx: &mut ViewContext<TerminalView>) -> impl IntoElement {
        cx.focus(&self.focus_handle);
        let content = self.output.read(cx).content.lock().unwrap();

        div()
            .pt_5()
            .track_focus(&self.focus_handle)
            .on_key_down(
                cx.listener(|this, event: &KeyDownEvent, _cx| match event.keystroke.key.as_str() {
                    "enter" => this.send_input("\r"),
                    "backspace" => this.send_input("\x7f"),
                    "space" => this.send_input(" "),
                    "t" => {
                        if event.keystroke.modifiers.control {
                            this.toggle_ansi();
                            _cx.notify();
                        } else {
                            this.send_input("t");
                        }
                    }
                    key if key.len() == 1 => this.send_input(key),
                    _ => {}
                }),
            )
            .size_full()
            .bg(rgb(0x282c34))
            .text_color(rgb(0xabb2bf))
            .font(font("monospace"))
            .child(div().size_full().p_2().items_start().child(div().flex_col().children(
                content.iter().rev().take(34).rev().map(|bytes| {
                    let line = match self.strip_ansi {
                        true => String::from_utf8(strip_ansi_escapes::strip(bytes)).unwrap(),
                        false => String::from_utf8_lossy(bytes).to_string(),
                    };
                    div().child(format!("{}", line))
                }),
            )))
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

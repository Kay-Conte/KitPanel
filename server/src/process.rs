use std::{
    collections::VecDeque,
    io::{self, BufRead, BufReader, Write},
    process::{Child, ChildStdout},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, RwLock,
    },
};

const SIZE: usize = 20;

pub struct Process {
    child: Option<Child>,
    pub console: Console,
    exit: Arc<AtomicBool>,
}

impl Process {
    pub fn new(mut child: Child) -> Self {
        let exit = Arc::new(AtomicBool::from(false));

        let stdout = child.stdout.take().expect("No stdout in Child");

        let console = Console::new(stdout, exit.clone());

        Self {
            child: Some(child),
            console,
            exit,
        }
    }

    pub fn send(&mut self, input: String) {
        self.console.write(input.clone());

        let Some(child) = &mut self.child else {
            return;
        };

        let _ = child.stdin.as_mut().unwrap().write_all(input.as_bytes());
    }

    pub fn insert(&mut self, mut child: Child) {
        self.exit.store(false, Ordering::Relaxed);

        let stdout = child.stdout.take().expect("No stdout in Child");

        self.child = Some(child);

        self.console.spawn(stdout, self.exit.clone());
    }

    pub fn is_alive(&self) -> bool {
        !self.exit.load(Ordering::Relaxed)
    }

    pub fn kill(&mut self) -> io::Result<()> {
        let Some(mut child) = self.child.take() else {
            return Ok(());
        };

        child.kill()
    }
}

pub struct Console {
    buf: Arc<RwLock<VecDeque<String>>>,
}

impl Console {
    fn new(source: ChildStdout, exit: Arc<AtomicBool>) -> Self {
        let buf = Arc::new(RwLock::new(VecDeque::new()));

        let console = Self { buf };

        console.spawn(source, exit);

        console
    }

    fn write(&self, input: String) {
        let mut buf = self.buf.write().unwrap();

        buf.push_front(input);

        if buf.len() > SIZE {
            buf.pop_back();
        }
    }

    pub fn spawn(&self, source: ChildStdout, exit: Arc<AtomicBool>) {
        let buf = self.buf.clone();

        std::thread::spawn(move || handle_stdout(buf, source, exit));
    }

    pub fn inner(&self) -> Vec<String> {
        self.buf.read().unwrap().clone().into()
    }
}

fn handle_stdout(buf: Arc<RwLock<VecDeque<String>>>, stdout: ChildStdout, sender: Arc<AtomicBool>) {
    let reader = BufReader::new(stdout);

    for line in reader.lines() {
        match line {
            Ok(line) => {
                let mut buf = buf.write().unwrap();

                buf.push_front(line);

                if buf.len() > SIZE {
                    buf.pop_back();
                }
            }
            Err(_) => break,
        }
    }

    sender.store(true, Ordering::Relaxed)
}

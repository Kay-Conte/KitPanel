use std::{
    collections::VecDeque,
    io::{self, BufRead, BufReader, Write},
    process::{Child, ChildStdout},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, RwLock,
    },
};

const BUFFER_SIZE: usize = 20;

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

    pub fn send(&mut self, input: String, user: Option<String>) {
        let Some(child) = &mut self.child else {
            return;
        };

        let e = child
            .stdin
            .as_mut()
            .unwrap()
            .write_all(format!("{}\n", input).as_bytes());

        println!("{e:?}");

        let display = match user {
            Some(user) => format!("[KitPanel - {}] {}", user, input),
            None => format!("[KitPanel] {}", input),
        };

        self.console.buf.write().unwrap().insert(display);
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

#[derive(Debug)]
pub struct Buffer {
    buf: VecDeque<String>,
    max: usize,
}

impl Buffer {
    fn new(max: usize) -> Self {
        Self {
            buf: VecDeque::new(),
            max,
        }
    }

    fn get(&self) -> &VecDeque<String> {
        &self.buf
    }

    fn insert(&mut self, content: String) {
        self.buf.push_front(content);

        if self.buf.len() > self.max {
            self.buf.pop_back();
        }
    }
}

pub struct Console {
    buf: Arc<RwLock<Buffer>>,
}

impl Console {
    fn new(source: ChildStdout, exit: Arc<AtomicBool>) -> Self {
        let buf = Arc::new(RwLock::new(Buffer::new(BUFFER_SIZE)));

        let console = Self { buf };

        console.spawn(source, exit);

        console
    }

    pub fn spawn(&self, source: ChildStdout, exit: Arc<AtomicBool>) {
        let buf = self.buf.clone();

        std::thread::spawn(move || handle_stdout(buf, source, exit));
    }

    pub fn inner(&self) -> Vec<String> {
        self.buf.read().unwrap().get().clone().into()
    }
}

fn handle_stdout(buf: Arc<RwLock<Buffer>>, stdout: ChildStdout, sender: Arc<AtomicBool>) {
    let reader = BufReader::new(stdout);

    for line in reader.lines() {
        match line {
            Ok(line) => {
                let mut buf = buf.write().unwrap();

                buf.insert(line);
            }
            Err(_) => break,
        }
    }

    sender.store(true, Ordering::Relaxed)
}

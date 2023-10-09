use std::io::{BufReader, Read, Write};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;

use crate::args;

pub struct Job {
    pub args: Vec<String>,
    pub return_code: Option<i32>,
    pub start_time: Option<std::time::SystemTime>,
    pub end_time: Option<std::time::SystemTime>,
}

impl Job {
    pub fn new(args: Vec<String>) -> Self {
        Job {
            args,
            return_code: None,
            start_time: None,
            end_time: None,
        }
    }

    pub fn args_printable(&self) -> String {
        let args = args::Args::from_vec(self.args.clone());
        args.printable()
    }

    pub fn start(&mut self) {
        self.start_time = Some(std::time::SystemTime::now());
    }

    pub fn end(&mut self) {
        self.end_time = Some(std::time::SystemTime::now());
    }
}

pub enum BuiltinCommand {
    Jobs,
}

pub enum ExecMessage {
    StdOut(String),
    StdErr(String),
    StdInQueue(mpsc::Sender<String>),
    BuiltinCommand(BuiltinCommand),
    JobDone(Job),
}

pub struct Execute {}

impl Execute {
    pub fn run(tx: mpsc::Sender<ExecMessage>, mut job: Job, stop_thread: Arc<AtomicBool>) {
        let tx_stdout = tx.clone();
        let tx_stderr = tx.clone();

        let send_stdout = move |s: String| {
            match tx_stdout.send(ExecMessage::StdOut(s)) {
                Ok(_) => {}
                Err(e) => {
                    println!("Error sending stdout: {}", e);
                }
            };
        };
        let send_stderr = move |s: String| {
            tx_stderr.send(ExecMessage::StdErr(s)).unwrap();
        };

        let send_loop = |reader: Box<dyn Read>, send: Box<dyn Fn(String)>| {
            let mut reader = reader;
            let mut buffer = [0; 1024];

            loop {
                match reader.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(size) => {
                        let bytes = &buffer[..size];
                        match std::str::from_utf8(bytes) {
                            Ok(s) => {
                                send(s.to_string());
                            }
                            Err(e) => {
                                if !e.valid_up_to() == 0 {
                                    let partial_string =
                                        std::str::from_utf8(&bytes[..e.valid_up_to()]).unwrap();
                                    send(partial_string.to_string());
                                }
                            }
                        }
                    }
                    Err(e) => {
                        println!("Error executing command: {}", e);
                        break;
                    }
                }
            }
        };

        job.start();
        if let Ok(mut child) = Command::new(&job.args[0])
            .args(&job.args[1..])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            if let Some(stdin) = child.stdin.take() {
                let mut stdin = stdin;
                let (tx_stdin, rx_stdin) = mpsc::channel::<String>();
                tx.send(ExecMessage::StdInQueue(tx_stdin)).unwrap();
                thread::spawn(move || {
                    for s in rx_stdin {
                        stdin.write_all(s.as_bytes()).unwrap();
                    }
                });
            };
            if let Some(stdout) = child.stdout.take() {
                let reader_stdout = BufReader::new(stdout);
                thread::spawn(move || send_loop(Box::new(reader_stdout), Box::new(send_stdout)));
            };
            if let Some(stderr) = child.stderr.take() {
                let reader_stderr = BufReader::new(stderr);
                thread::spawn(move || send_loop(Box::new(reader_stderr), Box::new(send_stderr)));
            };
            let mut return_code = 0;
            loop {
                match child.try_wait() {
                    Ok(Some(status)) => {
                        return_code = status.code().unwrap_or(1);
                        break;
                    }
                    Ok(None) => {}
                    Err(e) => {
                        println!("Error executing command: {}", e);
                        break;
                    }
                }

                // Check if we should stop
                if stop_thread.load(Ordering::SeqCst) {
                    println!("Stopping thread");
                    child.kill().unwrap();
                    tx.send(ExecMessage::StdErr(format!(
                        "Error: `{}` killed",
                        job.args_printable()
                    )))
                    .unwrap();
                    job.return_code = Some(1);
                    stop_thread.store(false, Ordering::SeqCst);
                } else {
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
            }
            job.return_code = Some(return_code);
        } else {
            tx.send(ExecMessage::StdErr(format!(
                "Error: `{}` not found",
                job.args_printable()
            )))
            .unwrap();
            job.return_code = Some(1);
        }
        job.end();
        tx.send(ExecMessage::JobDone(job)).unwrap_or_default();
    }
}

use std::io::{BufReader, Read};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;

pub enum ExecMessage {
    Command(String),
    StdOut(String),
    StdErr(String),
    ReturnCode(i32),
}

pub struct Execute {}

impl Execute {
    pub fn run(tx: mpsc::Sender<ExecMessage>, args: Vec<String>) {
        let tx_stdout = tx.clone();
        let tx_stderr = tx.clone();

        let send_stdout = move |s: String| {
            tx_stdout.send(ExecMessage::StdOut(s)).unwrap();
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

        if let Ok(mut child) = Command::new(&args[0])
            .args(&args[1..])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            if let Some(stdout) = child.stdout.take() {
                let reader_stdout = BufReader::new(stdout);
                thread::spawn(move || send_loop(Box::new(reader_stdout), Box::new(send_stdout)));
            };
            if let Some(stderr) = child.stderr.take() {
                let reader_stderr = BufReader::new(stderr);
                thread::spawn(move || send_loop(Box::new(reader_stderr), Box::new(send_stderr)));
            };
            let return_code = child.wait().unwrap().code().unwrap();
            tx.send(ExecMessage::ReturnCode(return_code)).unwrap();
        } else {
            tx.send(ExecMessage::StdErr("Error executing command".to_string()))
                .unwrap();
            tx.send(ExecMessage::ReturnCode(1)).unwrap();
        }
    }
}

use std::sync::atomic::AtomicBool;
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;

use crate::execute::{BuiltinCommand, ExecMessage, Execute, Job};
/*
1. cd: Change the current directory.
2. echo: Print arguments to the standard output.
3. exit: Exit the shell.
4. export: Set an environment variable.
5. pwd: Print the current working directory.
6. set: Set or unset shell options and positional parameters.
7. unset: Unset shell variables and functions.
8. exec: Replace the shell with the specified program.
9. shift: Shift positional parameters.
10. trap: Trap signals.
11. wait: Wait for a job to complete.
12. read: Read a line from the standard input.
13. test: Evaluate a conditional expression.
14. true: Return a successful result.
15. false: Return an unsuccessful result.
16. return: Exit from a shell function.
17. continue: Resume the next iteration of a loop.
18. break: Exit from a loop.
19. colon (:): No effect; the command does nothing.
20. dot (.): Source a shell script.
21. eval: Evaluate arguments as a shell command.
22. times: Print the accumulated user and system times for processes run from the
    shell.
23. ulimit: Get or set user limits.
24. umask: Get or set the file mode creation mask.
25. alias: Define or display aliases.
26. unalias: Remove alias definitions.
27. getopts: Parse positional parameters.
28. command: Run a command with a specific environment.
29. type: Display how each name would be interpreted if used as a command name.
30. jobs: List active jobs.
31. fg: Bring job to foreground.
32. bg: Send job to background.
33. kill: Send a signal to a job.
34. history: Command history.
35. help: Display help for a built-in command.
*/

pub struct Builtin {}

impl Builtin {
    pub fn run(tx: mpsc::Sender<ExecMessage>, job: Job) -> Option<Arc<AtomicBool>> {
        match job.args[0].as_str() {
            "cd" => Builtin::cmd_cd(tx, job),
            "jobs" => Builtin::cmd_jobs(tx, job),
            "yes" => Builtin::cmd_yes(tx, job),
            _ => Builtin::cmd_run_exec(tx, job),
        }
    }

    fn cmd_cd(tx: mpsc::Sender<ExecMessage>, mut job: Job) -> Option<Arc<AtomicBool>> {
        let v: Vec<String> = vec![];

        macro_rules! finish {
            ($return_code:expr, $output:expr) => {
                job.return_code = Some($return_code);
                if $output.len() > 0 {
                    tx.send(ExecMessage::StdOut($output.join("\n"))).unwrap();
                };
                job.end();
                tx.send(ExecMessage::JobDone(job)).unwrap();
            };
        }
        job.start();
        if job.args.len() == 1 {
            std::env::set_current_dir(std::env::var("HOME").unwrap()).unwrap();
            finish!(0, v);
        } else {
            let path = &job.args[1];
            let path = if path == "-" {
                std::env::var("OLDPWD").unwrap()
            } else {
                path.to_string()
            };
            let path = std::path::Path::new(&path);
            if path.exists() {
                std::env::set_current_dir(path).unwrap();
                finish!(0, v);
            } else {
                tx.send(ExecMessage::StdErr(format!(
                    "cd: {}: No such file or directory",
                    path.display()
                )))
                .unwrap();
                finish!(1, v);
            }
        }
        None
    }

    fn cmd_jobs(tx: mpsc::Sender<ExecMessage>, mut job: Job) -> Option<Arc<AtomicBool>> {
        job.start();
        tx.send(ExecMessage::BuiltinCommand(BuiltinCommand::Jobs))
            .unwrap();
        job.return_code = Some(0);
        None
    }

    fn cmd_yes(tx: mpsc::Sender<ExecMessage>, mut job: Job) -> Option<Arc<AtomicBool>> {
        job.start();
        tx.send(ExecMessage::StdOut("y".to_string())).unwrap();
        job.return_code = Some(0);
        None
    }

    fn cmd_run_exec(tx: mpsc::Sender<ExecMessage>, job: Job) -> Option<Arc<AtomicBool>> {
        let tx = tx.clone();
        let stop_thread = Arc::new(AtomicBool::new(false));
        let x = stop_thread.clone();
        thread::spawn(move || {
            Execute::run(tx, job, x);
        });
        Some(stop_thread)
    }

    pub fn jobs(tx: mpsc::Sender<ExecMessage>, jobs: &[Job]) {
        let mut v = vec![];
        for job in jobs {
            if job.end_time.is_none() {
                let t = std::time::SystemTime::now()
                    .duration_since(job.start_time.unwrap())
                    .unwrap();
                let time_human_readable = format!("{}.{:03}s", t.as_secs(), t.subsec_millis());
                v.push(format!(
                    "{} running for {}",
                    job.args.join(" "),
                    time_human_readable
                ));
            }
        }
        tx.send(ExecMessage::StdOut(v.join("\n"))).unwrap();
    }
}

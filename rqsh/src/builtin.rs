use std::sync::mpsc;
use std::thread;

use crate::execute::{ExecMessage, Execute, Job};
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

pub struct BuiltIn {}

impl BuiltIn {
    pub fn run(tx: mpsc::Sender<ExecMessage>, mut job: Job) {
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

        let v: Vec<String> = vec![];

        match job.args[0].as_str() {
            "cd" => {
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
            }
            "yes" => {
                job.start();
                let output = vec!["yes".to_string()];
                finish!(0, output);
            }
            _ => {
                let tx = tx.clone();
                thread::spawn(move || {
                    Execute::run(tx, job);
                });
            }
        };
    }
}

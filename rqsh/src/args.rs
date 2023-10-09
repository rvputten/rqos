pub struct Args {
    pub args: Vec<String>,
}

impl Args {
    pub fn new(s: &str) -> Self {
        let mut args = vec![];
        let mut arg = String::new();
        let mut in_quote = false;
        let mut escaped = false;

        for c in s.chars() {
            if escaped {
                arg.push(c);
                escaped = false;
                continue;
            }

            match c {
                '\\' => {
                    escaped = true;
                }
                '"' => {
                    in_quote = !in_quote;
                }
                ' ' if !in_quote => {
                    if !arg.is_empty() {
                        args.push(arg.clone());
                        arg.clear();
                    }
                }
                _ => {
                    arg.push(c);
                }
            }
        }

        if !arg.is_empty() {
            args.push(arg);
        }

        Args { args }
    }

    pub fn new_notrim(s: &str) -> Self {
        let mut args = Args::new(s);
        if s.ends_with(' ') {
            args.args.push(String::new());
        }
        args
    }

    pub fn from_vec(args: Vec<String>) -> Self {
        Args { args }
    }

    pub fn printable(&self) -> String {
        let mut s = String::new();
        for arg in &self.args {
            let mut arg_quotes = String::new();
            for c in arg.chars() {
                if c == '"' {
                    arg_quotes.push('\\');
                }
                arg_quotes.push(c);
            }

            if arg_quotes.contains(' ') {
                s.push_str(&format!("\"{}\" ", arg_quotes));
            } else {
                s.push_str(&format!("{} ", arg_quotes));
            }
        }
        s.pop();
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_args_new() {
        let args = Args::new("ls -l");
        assert_eq!(args.args, vec!["ls", "-l"]);
    }

    #[test]
    fn test_args_new_notrim() {
        let args = Args::new_notrim("ls -l ");
        assert_eq!(args.args, vec!["ls", "-l", ""]);
    }

    #[test]
    fn test_args_from_vec() {
        let args = Args::from_vec(vec!["ls".to_string(), "-l".to_string()]);
        assert_eq!(args.args, vec!["ls", "-l"]);
    }

    #[test]
    fn test_args_printable() {
        let args = Args::new("ls -l");
        assert_eq!(args.printable(), "ls -l");
    }

    #[test]
    fn test_args_printable_with_quotes() {
        let args = Args::new("ls -l \"foo bar\"");
        assert_eq!(args.printable(), "ls -l \"foo bar\"");
    }

    #[test]
    fn test_args_printable_with_quotes_and_escaped_quotes() {
        let args = Args::new("ls -l \"foo \\\"bar\\\"\"");
        assert_eq!(args.printable(), "ls -l \"foo \\\"bar\\\"\"");
    }
}

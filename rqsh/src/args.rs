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
}

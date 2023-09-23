#![allow(dead_code)]
struct Glob {
    source: Vec<String>,
}

impl Glob {
    pub fn new(source: &[&str]) -> Self {
        Self {
            source: source.iter().map(|s| s.to_string()).collect(),
        }
    }

    fn matches(pattern: &str, to_match: &str) -> bool {
        let mut pattern_chars = pattern.chars().peekable();
        let mut to_match_chars = to_match.chars().peekable();
        loop {
            match (pattern_chars.peek(), to_match_chars.peek()) {
                (Some('?'), Some(_)) => {
                    pattern_chars.next();
                    to_match_chars.next();
                }
                (Some('*'), _) => {
                    pattern_chars.next();
                    if pattern_chars.peek().is_none() {
                        return true;
                    }
                    let remaining_pattern: String = pattern_chars.collect();
                    while let Some(_) = to_match_chars.peek() {
                        let remaining_to_match: String = to_match_chars.clone().collect();
                        if Self::matches(&remaining_pattern, &remaining_to_match) {
                            return true;
                        }
                        to_match_chars.next();
                    }
                    return false;
                }
                (Some(c1), Some(c2)) => {
                    if c1 != c2 {
                        return false;
                    }
                    pattern_chars.next();
                    to_match_chars.next();
                }
                (Some(_), None) => return false,
                (None, Some(_)) => return false,
                (None, None) => return true,
            }
        }
    }

    pub fn glob(&self, pattern: &str) -> Vec<String> {
        let mut result = Vec::new();
        for s in &self.source {
            if Self::matches(pattern, s) {
                result.push(s.clone());
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches() {
        assert!(Glob::matches("abc", "abc"));
        assert!(!Glob::matches("abc", "ab"));
        assert!(!Glob::matches("abc", "abcd"));
        assert!(Glob::matches("abc*", "abc"));
        assert!(Glob::matches("abc*", "abcd"));
        assert!(Glob::matches("abc*", "abcde"));
        assert!(Glob::matches("abc*e", "abcde"));
        assert!(!Glob::matches("abc*f", "abcde"));
        assert!(Glob::matches("*abc", "abc"));
        assert!(!Glob::matches("*abc", "abe"));

        assert!(Glob::matches("a?c", "abc"));
    }

    #[test]
    fn test_glob() {
        let glob = Glob::new(&["abc", "abcd", "abcde"]);
        let empty: Vec<&str> = Vec::new();
        assert_eq!(glob.glob("abc"), vec!["abc"]);
        assert_eq!(glob.glob("abc*"), vec!["abc", "abcd", "abcde"]);
        assert_eq!(glob.glob("abc*e"), vec!["abcde"]);
        assert_eq!(glob.glob("abc*f"), empty);
    }
}

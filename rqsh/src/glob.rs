pub struct Glob {
    source: Vec<String>,
    parent: String,
}

impl Glob {
    pub fn from_vec_string(source: Vec<String>, parent: &str) -> Self {
        Self {
            source,
            parent: parent.to_string(),
        }
    }

    pub fn from_path(path: &str) -> Result<Self, std::io::Error> {
        let mut source = Vec::new();
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            if let Ok(name) = entry.file_name().into_string() {
                source.push(name);
            } else {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "invalid file name",
                ));
            }
        }
        source.sort();
        Ok(Glob::from_vec_string(source, path))
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
                    while to_match_chars.peek().is_some() {
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

    pub fn match_path_single(&self, pattern: &str) -> Vec<String> {
        let mut result = Vec::new();
        for s in &self.source {
            if Self::matches(pattern, s) {
                result.push(s.clone());
            }
        }
        result
    }

    // pattern can contain any number of '/', '?' and '*', not allowed are '[]{}"\' etc.
    // prerequisites:
    // - self.source contains all files and directories in the directory to be searched
    //   - e.g. ["file1", "file2", "subdir1", "subdir2"], but not ["subdir2/file3"] during the
    //     initial call
    // - when called recursively by self, self.source contains deeper levels, but always the same
    //   level
    //   - e.g. ["subdir1/file3", "subdir2/file4", "subdir2/subsubdir1"]
    //   - but not ["file1", "subdir2/subdir3/file5"]
    // - pattern starts with the pattern to be matched
    pub fn match_path_multiple(&self, pattern: &str) -> Vec<String> {
        if let Some(stripped) = pattern.strip_prefix('/') {
            let glob = Glob::from_path("/").unwrap();
            if stripped.is_empty() {
                vec![]
            } else {
                glob.match_path_multiple(stripped)
            }
        } else {
            let mut parts = pattern.splitn(2, '/');
            let here = parts.next().unwrap();
            let matched_here = self.match_path_single(here);

            let path_join = |s1: &str, s2: &str| match s1 {
                "." => s2.to_string(),
                "/" => format!("/{}", s2),
                _ => format!("{}/{}", s1, s2),
            };

            // if there is a '/' in the pattern, we need to match the rest of the pattern in the
            // matched files
            if let Some(remaining_match_path) = parts.next() {
                // what we have now:
                // - list of files and directories in the current directory that match the pattern
                // - the rest of the pattern to be matched
                // - in self.source, we have
                // what we need to do:
                // - go into the directory (if it is a directory) and match the rest of the pattern
                //   recursively
                let mut result = Vec::new();
                for entry in matched_here {
                    let path = path_join(&self.parent, &entry);
                    let path = std::path::Path::new(&path);
                    if path.is_dir() {
                        let entry = path_join(&self.parent, &entry);
                        match Glob::from_path(&entry) {
                            Ok(glob) => {
                                let matched = glob.match_path_multiple(remaining_match_path);
                                for m in matched {
                                    result.push(m);
                                }
                            }
                            Err(e) => {
                                eprintln!("error: {}", e);
                            }
                        };
                    }
                }
                result
            } else if self.parent == "." {
                matched_here
            } else {
                matched_here
                    .iter()
                    .map(|s| path_join(&self.parent, s))
                    .collect()
            }
        }
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
        let strings = ["abc", "abcd", "abcde"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let glob = Glob::from_vec_string(strings, ".");
        let empty: Vec<&str> = Vec::new();
        assert_eq!(glob.match_path_single("abc"), vec!["abc"]);
        assert_eq!(glob.match_path_single("abc*"), vec!["abc", "abcd", "abcde"]);
        assert_eq!(glob.match_path_single("abc*e"), vec!["abcde"]);
        assert_eq!(glob.match_path_single("abc*f"), empty);
    }

    fn setup_match_path_multiple() -> String {
        // create a test dir with mktemp
        let r = std::process::Command::new("mktemp")
            .arg("-d")
            .output()
            .expect("failed to execute process");
        let dir = String::from_utf8(r.stdout).unwrap();
        let dir = dir.trim();
        let new_dirs = ["/subdir1", "/subdir2", "/subdir2/subsubdir1"];
        let new_files = [
            "/file1",
            "/file2",
            "/subdir1/file3",
            "/subdir2/file4",
            "/subdir2/subsubdir1/file5",
        ];

        for d in &new_dirs {
            let path = format!("{}{}", dir, d);
            std::fs::create_dir_all(path).unwrap();
        }

        for f in &new_files {
            let path = format!("{}{}", dir, f);
            std::fs::File::create(path).unwrap();
        }

        dir.to_string()
    }

    fn cleanup_match_path_multiple(dir: &str) {
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn test_match_path_multiple() {
        let dir = setup_match_path_multiple();
        // go to directory
        let _ = std::env::set_current_dir(&dir);
        let glob = Glob::from_path(".").unwrap();
        let empty: Vec<&str> = Vec::new();
        let assert_sorted = |pattern: &str, expected: Vec<&str>| {
            let mut result = glob.match_path_multiple(pattern);
            result.sort();
            println!("actual  : {:?}", result);
            println!("expected: {:?}", expected);
            assert_eq!(result, expected);
        };

        assert_sorted("file*", vec!["file1", "file2"]);
        assert_sorted("subdir*", vec!["subdir1", "subdir2"]);
        assert_sorted("subdir1/*", vec!["subdir1/file3"]);
        assert_sorted("subdir?/f*", vec!["subdir1/file3", "subdir2/file4"]);
        assert_sorted("subdir2/*", vec!["subdir2/file4", "subdir2/subsubdir1"]);
        assert_sorted("subdir2/subsubdir1/*", vec!["subdir2/subsubdir1/file5"]);
        assert_sorted("subdir2/subsubdir1/file5", vec!["subdir2/subsubdir1/file5"]);
        assert_sorted("subdir2/subsubdir1/file6", empty);
        assert_sorted("/tmp", vec!["/tmp"]);
        assert_sorted(&dir, vec![&dir]);
        assert_sorted(
            &format!("{}/subdir*", dir),
            vec![&format!("{}/subdir1", dir), &format!("{}/subdir2", dir)],
        );
        cleanup_match_path_multiple(&dir);
    }
}

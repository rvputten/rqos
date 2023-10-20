use std::io::{Read, Write};

pub struct Config {
    rqos_data_home: String,
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl Config {
    pub fn new() -> Self {
        let xdg_data_home = std::env::var("XDG_DATA_HOME")
            .unwrap_or_else(|_| format!("{}/.local/share", env!("HOME")));
        Self {
            rqos_data_home: format!("{}/rqos", xdg_data_home),
        }
    }

    pub fn get_filename(&self, filename: &str) -> String {
        format!("{}/{}", self.rqos_data_home, filename)
    }

    pub fn get_file(&self, filename: &str) -> Result<String, std::io::Error> {
        let path = self.get_filename(filename);
        let file = std::fs::File::open(path)?;
        let mut buf_reader = std::io::BufReader::new(file);
        let mut contents = String::new();
        buf_reader.read_to_string(&mut contents)?;
        Ok(contents)
    }

    pub fn write_file(&self, filename: &str, content: &str) -> Result<(), std::io::Error> {
        let path = self.get_filename(filename);
        let mut file = std::fs::File::create(path)?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }

    pub fn get_resource_path(file: &str) -> String {
        if cfg!(debug_assertions) {
            let pwd = std::env::current_dir().unwrap();
            if pwd.ends_with("rqos") {
                format!("resources/{}", file)
            } else {
                format!("../resources/{}", file)
            }
        } else {
            Config::new().get_filename(&format!("resources/{}", file))
        }
    }
}

pub fn get_resource_path(file: &str) -> String {
    if cfg!(debug_assertions) {
        let pwd = std::env::current_dir().unwrap();
        if pwd.ends_with("rqos") {
            format!("resources/{}", file)
        } else {
            format!("../resources/{}", file)
        }
    } else {
        format!("{}/.local/share/rqos/resources/{}", env!("HOME"), file)
    }
}

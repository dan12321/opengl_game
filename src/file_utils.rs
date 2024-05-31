use std::{fs, path::PathBuf};

pub fn get_level_file(level_dir: &PathBuf, file_type: &str) -> PathBuf {
    let mut dir = fs::read_dir(level_dir).unwrap();
    let file = dir
        .find(|f| {
            f.as_ref()
                .unwrap()
                .file_name()
                .to_str()
                .unwrap()
                .ends_with(file_type)
        })
        .unwrap()
        .unwrap()
        .file_name();
    level_dir.join(file)
}

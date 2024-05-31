use std::{fs::OpenOptions, io::Read, path::PathBuf};

pub struct Map {
    pub bpm: f32,
    pub subdivisions: f32,
    pub beats: Vec<(bool, bool, bool)>,
}

impl Map {
    pub fn from_file(filepath: &PathBuf) -> Self {
        let mut file = OpenOptions::new().read(true).open(filepath).unwrap();

        let mut buf = String::new();
        file.read_to_string(&mut buf).unwrap();
        let mut lines = buf.lines();
        let metadata = lines.next().unwrap();
        let metadata_parts: Vec<f32> = metadata.split(",").map(|s| s.parse().unwrap()).collect();
        let beats: Vec<(bool, bool, bool)> = lines.map(|s| parse_map_line(s)).collect();

        Map {
            bpm: metadata_parts[0],
            subdivisions: metadata_parts[1],
            beats,
        }
    }
}

fn parse_map_line(s: &str) -> (bool, bool, bool) {
    let mut result = [false; 3];
    for i in 0..3 {
        result[i] = s[i..i + 1] == *"#";
    }
    (result[0], result[1], result[2])
}

use std::{
    error::Error,
    fmt::Display,
    fs::OpenOptions,
    io::Read,
    num::ParseFloatError,
};

use anyhow::{Result, Context};

use super::manager::Loadable;

#[derive(Debug)]
pub struct Map {
    pub version: u64,
    pub bpm: f32,
    pub subdivisions: f32,
    pub start_offset: f32,
    pub music: String,
    pub beats: Vec<(bool, bool, bool)>,
}

impl Loadable for Map {
    type Output = Self;
    fn load(file: &str) -> Result<Self> {
        let mut file = OpenOptions::new().read(true).open(file)?;

        let mut buf = String::new();
        file.read_to_string(&mut buf)?;
        let mut lines = buf.lines().filter(|l| !l.starts_with("//"));
        let version: u64 = lines.next()
            .ok_or(MapError::MissingVersion)?
            .trim()
            .parse()
            .context("Parsing Map Version")?;
        let metadata = lines.next().ok_or(MapError::MissingMetadata)?;
        let metadata_parts: Result<Vec<f32>, ParseFloatError> = metadata.split(",")
            .map(|s| s.parse())
            .collect();
        let metadata_parts = metadata_parts.context("Parsing Map Metadata")?;
        if metadata_parts.len() != 3 {
            return Err(MapError::BadMetadata(metadata_parts.len()).into());
        }
        let music = lines.next().ok_or(MapError::MissingSongData)?;
        let beats: Vec<(bool, bool, bool)> = lines.map(|s| parse_map_line(s)).collect();

        Ok(Map {
            version,
            bpm: metadata_parts[0],
            subdivisions: metadata_parts[1],
            start_offset: metadata_parts[2],
            music: music.to_string(),
            beats,
        })
    }
}

fn parse_map_line(s: &str) -> (bool, bool, bool) {
    let mut result = [false; 3];
    for i in 0..3 {
        result[i] = s[i..i + 1] == *"#";
    }
    (result[0], result[1], result[2])
}

#[derive(Debug)]
enum MapError {
    MissingVersion,
    MissingMetadata,
    MissingSongData,
    BadMetadata(usize),
}

impl Display for MapError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingVersion => write!(f, "Missing Version"),
            Self::MissingMetadata => write!(f, "Missing Metadata"),
            Self::MissingSongData => write!(f, "Missing Song Data"),
            Self::BadMetadata(n) => write!(f, "Found {} metadata arguments", n)
        }
    }
}

impl Error for MapError {}
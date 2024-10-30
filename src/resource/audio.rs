use std::{
    error::Error,
    fmt::Display,
    fs::OpenOptions,
    io::Read,
    path::PathBuf,
};

use anyhow::Result;
use tracing::debug;

const WAV_HEADER_SIZE: usize = 44;

#[derive(Debug)]
pub struct Wav {
    // If memory becomes an issue a VecDequeue should be used
    pub samples: Vec<f64>,
    pub sample_rate: u32,
    pub sample_size: u32,
}

impl Wav {
    pub fn new(path: &PathBuf) -> Result<Self>{
        let mut file = OpenOptions::new().read(true).open(&path)?;

        let mut file_header: [u8; WAV_HEADER_SIZE] = [0; WAV_HEADER_SIZE];
        let bytes_read = file.read(&mut file_header)?;
        if bytes_read < file_header.len() {
            return Err(ParseWavError::HeaderTooSmall(bytes_read).into());
        }

        let correct_subtype = &file_header[0..4] == b"RIFF";
        let correct_filetype = &file_header[8..12] == b"WAVE";
        let filesize_bytes = read_from_buffer(&file_header[4..8]);
        let filesize = u32::from_le_bytes(filesize_bytes);
        let format_bytes: [u8; 2] = read_from_buffer(&file_header[20..22]);
        let format = u16::from_le_bytes(format_bytes);
        let channel_bytes: [u8; 2] = read_from_buffer(&file_header[22..24]);
        let channel = u16::from_le_bytes(channel_bytes);
        let sample_rate_bytes: [u8; 4] = read_from_buffer(&file_header[24..28]);
        let sample_rate = u32::from_le_bytes(sample_rate_bytes);
        let bytes_per_second_bytes: [u8; 4] = read_from_buffer(&file_header[28..32]);
        let bytes_per_second = u32::from_le_bytes(bytes_per_second_bytes);
        let bits_per_sample_bytes: [u8; 2] = read_from_buffer(&file_header[34..36]);
        let bits_per_sample = u16::from_le_bytes(bits_per_sample_bytes);
        let data_size_bytes: [u8; 4] = read_from_buffer(&file_header[40..44]);
        let data_size = u32::from_le_bytes(data_size_bytes);

        let samples = unsafe {
            let mut sample_buffer: Vec<u8> = Vec::with_capacity(data_size as usize);
            let ds = data_size  as usize;
            sample_buffer.set_len(ds);
            // TODO: Fix interrupt handling. n often less than ds. 
            let _ = file.read(&mut sample_buffer)?;
            // if n != ds {
            //     return Err(ParseWavError::DataSizeInconsistent(n, ds).into());
            // }
            parse_samples(&sample_buffer)
        };
        debug!(
            correct_subtype = correct_subtype,
            correct_filetype = correct_filetype,
            filesize = filesize,
            format = format,
            channel = channel,
            sample_rate = sample_rate,
            bytes_per_second = bytes_per_second,
            bits_per_sample = bits_per_sample,
            data_size = data_size,
            "open wav file",
        );
        // assume 1 chanel for now
        Ok(Wav {
            samples,
            sample_rate,
            sample_size: bytes_per_second / sample_rate,
        })
    }
}

fn read_from_buffer<const T: usize>(slice: &[u8]) -> [u8; T] {
    let mut bytes = [0; T];
    bytes.clone_from_slice(slice);
    bytes
}

fn parse_samples(bytes: &[u8]) -> Vec<f64> {
    let num_samples = bytes.len() / 2;
    let mut samples = Vec::with_capacity(num_samples);
    for i in 0..num_samples {
        let pos = i * 2;
        let byte: [u8; 2] = read_from_buffer(&bytes[pos..pos + 2]);
        let int = i16::from_le_bytes(byte);
        samples.push(int as f64);
    }
    samples
}

#[derive(Debug)]
enum ParseWavError {
    HeaderTooSmall(usize),
    DataSizeInconsistent(usize, usize),
}

impl Display for ParseWavError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::HeaderTooSmall(bytes) => write!(f, "Expected {} bytes in header but only found {}", WAV_HEADER_SIZE, bytes),
            Self::DataSizeInconsistent(bytes, expected) => write!(f, "Expected {} bytes in data but found {}", expected, bytes),
        }
    }
}

impl Error for ParseWavError {}
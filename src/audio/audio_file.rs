use std::{collections::VecDeque, fs::{File, OpenOptions}, io::{BufReader, Read}, path::PathBuf, sync::RwLock};

use tracing::debug;

#[derive(Debug)]
pub struct Wav {
    // TODO: Rethink this structure, starting to feel like a recipe for
    // deadlocks. Maybe get_next_audio_value can be made dumber and just
    // read off of a channel being populated by the mixer?
    pub reader: RwLock<BufReader<File>>,
    pub samples: RwLock<VecDeque<f64>>,
    pub sample_rate: u32,
    pub sample_size: u32,
    pub chunk_size: usize,
    pub ended: bool,
}

impl Wav {
    pub fn new(path: &PathBuf) -> Self {
        let file = OpenOptions::new().read(true).open(&path).unwrap();
        let mut reader = BufReader::new(file);

        let mut file_header: [u8; 44] = [0; 44];
        reader.read(&mut file_header).unwrap();

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

        let mut sample_buffer: Vec<u8> = Vec::with_capacity(data_size as usize);
        unsafe {
            sample_buffer.set_len(data_size as usize);
        }
        let num = reader.read(&mut sample_buffer).unwrap();
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
            data_read = num,
            audio_bytes = format!("{:?}", &sample_buffer[0..20]),
            fsb = format!("{:?}", filesize_bytes),
            file_header = format!("{:?}", file_header),
            "open wav file",
        );
        // assume 1 chanel for now
        // let samples = parse_samples(&sample_buffer);
        let chunk_size = 1024;
        let samples = VecDeque::with_capacity(chunk_size * 2);
        Wav {
            reader: RwLock::new(reader),
            samples: RwLock::new(samples),
            sample_rate,
            sample_size: bytes_per_second / sample_rate,
            chunk_size,
            ended: false,
        }
    }

    pub fn get_next(&self) -> f64 {
        let samples_len = {
            self.samples.read().unwrap().len()
        };
        if samples_len < self.chunk_size / 2 {
            let mut next_chunk = self.parse_chunk();
            self.samples.write()
                .unwrap()
                .append(&mut next_chunk);
            // self.ended = next_chunk.len() == 0;
        }
        self.samples.write()
            .unwrap()
            .pop_front()
            .unwrap_or(0.0)
    }

    fn parse_chunk(&self) -> VecDeque<f64> {
        let num_bytes = self.chunk_size * 2;
        let mut bytes = vec![0; num_bytes];
        let n = self.reader.write().unwrap().read(&mut bytes).unwrap();
        let mut res = VecDeque::with_capacity(self.chunk_size);
        for i in 0..(n / 2) {
            let pos = i * 2;
            let byte: [u8; 2] = read_from_buffer(&bytes[pos..pos + 2]);
            let int = i16::from_le_bytes(byte);
            res.push_back(int as f64);
        }
        res
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


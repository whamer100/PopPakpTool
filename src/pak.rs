use std::fmt::Formatter;
use std::fs::{create_dir_all, File};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path};
use std::time::{SystemTime, Duration};
use logma::{debug, fatal, info, warn};
use crate::iohelper::IoHelper;

const EPOCH_AS_FILETIME: u64 = 116444736000000000;
pub const PAK_MAGIC: u32 = 0xBAC04AC0;
pub const FILEFLAGS_END: u8 = 0x80;

#[derive(Debug)]
pub enum PakError {
    ParseError(String)
}

#[derive(Debug)]
pub struct PakRecord {
    pub fname: String,
    pub start_pos: u32,
    pub size: u32,
    pub datetime: u64,  // todo: actually parse this
}

#[derive(Default)]
pub struct PakFile {
    pub is_pak: bool,
    pub xor: u8,
    pub version: u32,
    pub records: Vec<PakRecord>,
    pak_data: Vec<u8>,
    data_pos: u64
}

impl std::fmt::Debug for PakFile {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PakFile")
            .field("version", &self.version)
            .field("is_pak", &self.is_pak)
            .field("xor", &self.xor)
            .field("records", &format_args!("[ PakRecord count: {} ]", self.records.len()))
            .finish()
    }
}

impl PakFile {
    pub fn new() -> Self { PakFile::default() }

    /// Parse Pak file from a given path
    ///
    /// Returns a PakError on fatal exception during parsing
    pub fn parse(&mut self, infile: &Path) -> Result<(), PakError> {
        let mut f = File::open(infile).unwrap();

        f.seek(SeekFrom::End(0)).unwrap();
        let file_size = f.stream_position().unwrap();

        f.seek(SeekFrom::Start(0)).unwrap();
        let mut raw_data = vec![0u8; file_size as usize];
        self.pak_data.resize(file_size as usize, 0u8);

        f.read_exact(&mut raw_data).unwrap();

        let mut r = IoHelper::new(&raw_data);

        // let mut c = Cursor::new(&raw_data);

        let magic: u32 = r.read_u32();
        if magic == PAK_MAGIC {
            self.is_pak = true;
            debug!("test: {}", magic);
        }
        else if magic ^ 0xF7F7F7F7 == PAK_MAGIC {
            self.is_pak = true;
            self.xor = 0xF7;
            debug!("test: {}", magic);
        }

        if self.is_pak {
            info!("PopCap Pak file format found.");
            for (i, x) in raw_data.iter().enumerate() {
                self.pak_data[i] = x ^ self.xor
            };

            let mut r = IoHelper::new(&self.pak_data);

            let magic: u32 = r.read_u32();
            assert_eq!(magic, PAK_MAGIC);

            self.version = r.read_u32();

            info!("Pak version: {}", self.version);
            if self.version > 0 {
                warn!("Pak version unexpected! Errors may occur.");
            }
            // TODO:
            //  - impl: [in pack] scratch that, reverse it

            PakFile::read_records(&mut self.records, &mut r);
            self.data_pos = r.tell();
        }
        else {
            return Err(PakError::ParseError(String::from("Format not identified.")))
        }
        Ok(())
    }

    fn read_records(records: &mut Vec<PakRecord>, r: &mut IoHelper) {
        let mut pos: u32 = 0;
        info!("Parsing file table...");
        loop {
            let flags = r.read_u8();
            if flags & FILEFLAGS_END > 0 {
                break
            }
            let fname = r.read_str();

            let src_size = r.read_u32();
            let file_time = r.read_u64();
            
            let record = PakRecord {
                fname,
                start_pos: pos,
                size: src_size,
                datetime: file_time,
            };

            records.push(record);
            pos += src_size;
        }
        let count = records.len();
        info!("Parsed {} files.", count);
    }

    pub fn dump_files(&self, outdir: &Path) {
        let mut r = IoHelper::new(&self.pak_data);
        info!("Writing files to \"{}\".", outdir.to_str().unwrap());
        for x in &self.records {
            r.seek(x.start_pos as u64 + self.data_pos);
            let data = r.read_bytes(x.size as usize);
            let target_file = Path::join(outdir, &x.fname);
            if target_file.exists() {
                std::fs::remove_file(&target_file).unwrap();
            }
            create_dir_all(target_file.parent().unwrap()).unwrap();

            let mut new_file = File::create(target_file).unwrap();
            new_file.write_all(data.as_slice()).unwrap();
            let dt = PakFile::filetime_to_datetime(x.datetime);
            new_file.set_modified(SystemTime::UNIX_EPOCH + Duration::from_micros(dt)).unwrap();
        }
    }

    fn filetime_to_datetime(ft: u64) -> u64 {
        return (ft - EPOCH_AS_FILETIME) / 10
    }

    fn datetime_to_filetime(dt: u64) -> u64 {
        return (dt * 10) + EPOCH_AS_FILETIME
    }
}
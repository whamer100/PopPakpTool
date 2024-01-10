use std::io::{Cursor, Read};
use byteorder::{LittleEndian, ReadBytesExt};

pub struct IoHelper<'a> {
    c: Cursor<&'a Vec<u8>>
}

impl IoHelper<'_> {
    pub fn new(buf: &Vec<u8>) -> IoHelper<'_> { IoHelper{ c: Cursor::new(&buf) } }
    pub fn seek(&mut self, i: u64) {
        self.c.set_position(i);
    }
    pub fn tell(&mut self) -> u64 {
        self.c.position()
    }
    pub fn read_u8(&mut self) -> u8 {
        self.c.read_u8().unwrap()
    }
    pub fn read_u16(&mut self) -> u16 {
        self.c.read_u16::<LittleEndian>().unwrap()
    }
    pub fn read_u32(&mut self) -> u32 {
        self.c.read_u32::<LittleEndian>().unwrap()
    }
    pub fn read_u64(&mut self) -> u64 {
        self.c.read_u64::<LittleEndian>().unwrap()
    }
    pub fn read_bytes(&mut self, size: usize) -> Vec<u8> {
        let mut buf = vec![0u8; size];
        self.c.read_exact(&mut buf).unwrap();
        return buf
    }
    pub fn read_str(&mut self) -> String {
        let len = self.read_u8();
        let buf = self.read_bytes(len as usize);
        return String::from_utf8(buf).unwrap();
    }
}
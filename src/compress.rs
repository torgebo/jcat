//! Compression functionality
use flate2::read::{GzDecoder, ZlibDecoder};
use flate2::write::{GzEncoder, ZlibEncoder};
use flate2::Compression;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq)]
pub enum ContentType {
    Other,
    Json(CType),
}

#[derive(Debug, PartialEq)]
pub enum CType {
    Raw,
    Gzip,
    Snappy,
    Zlib,
}

impl From<&Path> for ContentType {
    fn from(p: &Path) -> Self {
        ContentType::from(&p.to_path_buf())
    }
}

impl From<&PathBuf> for ContentType {
    fn from(p: &PathBuf) -> Self {
        match p.file_name() {
            None => ContentType::Other,
            Some(osstr) => match osstr.to_str() {
                None => ContentType::Other,
                Some(s) if s.ends_with(".json") => ContentType::Json(CType::Raw),
                Some(s) if s.ends_with(".json.gz") => ContentType::Json(CType::Gzip),
                Some(s) if s.ends_with(".json.sz") => ContentType::Json(CType::Snappy),
                Some(s) if s.ends_with(".json.zz") => ContentType::Json(CType::Zlib),
                _ => ContentType::Other,
            },
        }
    }
}

impl CType {
    pub fn get_extension(&self) -> &str {
        match self {
            Self::Raw => "json",
            Self::Gzip => "json.gz",
            Self::Snappy => "json.sz",
            Self::Zlib => "json.zz",
        }
    }
}

/// Wrap `buf_reader` in the correct Read-er
/// to support chosen compression (or none)
pub fn read_from_ctype(buf_reader: BufReader<File>, ct: &CType) -> Box<dyn Read> {
    match ct {
        CType::Raw => Box::new(buf_reader),
        CType::Gzip => Box::new(GzDecoder::new(buf_reader)),
        CType::Snappy => Box::new(snap::read::FrameDecoder::new(buf_reader)),
        CType::Zlib => Box::new(ZlibDecoder::new(buf_reader)),
    }
}

/// Wrap `buf_writer` in the correct Write-er
/// to support chosen compression (or none)
pub fn write_from_ctype(buf_writer: BufWriter<File>, ct: &CType) -> Box<dyn Write> {
    match ct {
        CType::Raw => Box::new(buf_writer),
        CType::Gzip => Box::new(GzEncoder::new(buf_writer, Compression::default())),
        CType::Snappy => Box::new(snap::write::FrameEncoder::new(buf_writer)),
        CType::Zlib => Box::new(ZlibEncoder::new(buf_writer, Compression::default())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_type_from_path() {
        assert_eq!(
            ContentType::Json(CType::Raw),
            ContentType::from(Path::new("/tmp/1.json"))
        );
        assert_eq!(
            ContentType::Json(CType::Gzip),
            ContentType::from(Path::new("/tmp/1.json.gz"))
        );
        assert_eq!(
            ContentType::Json(CType::Snappy),
            ContentType::from(Path::new("/tmp/1.json.sz"))
        );
        assert_eq!(
            ContentType::Json(CType::Zlib),
            ContentType::from(Path::new("/tmp/1.json.zz"))
        );
        assert_eq!(ContentType::Other, ContentType::from(Path::new("/tmp/1")));
    }
}

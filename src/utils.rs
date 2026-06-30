use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

#[derive(Debug, PartialEq)]
pub enum FileType {
    Xlsx,
    Xls,
    Csv,
    Unknown,
}

pub fn detect_file_type<P: AsRef<Path>>(path: P) -> io::Result<FileType> {
    let path = path.as_ref();

    // Prefer the file extension for text formats: CSV/TSV have no magic bytes,
    // so extension is the most reliable signal.
    if let Some(ext) = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
    {
        match ext.as_str() {
            "csv" | "tsv" | "tab" => return Ok(FileType::Csv),
            "xlsx" | "xlsm" | "xlsb" | "ods" => return Ok(FileType::Xlsx),
            "xls" => return Ok(FileType::Xls),
            _ => {}
        }
    }

    let mut file = File::open(path)?;
    let mut buffer = [0u8; 8];

    // Read the first 8 bytes (covers both the ZIP and OLE2 magic-number lengths).
    let n = file.read(&mut buffer)?;

    // Too short to identify by magic number.
    if n < 8 {
        return Ok(FileType::Unknown);
    }

    // XLSX/ZIP magic number (PK..)
    if buffer[..4] == [0x50, 0x4B, 0x03, 0x04] {
        return Ok(FileType::Xlsx);
    }

    // XLS (OLE2/CFB) magic number (D0 CF 11 E0 A1 B1 1A E1)
    if buffer == [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1] {
        return Ok(FileType::Xls);
    }

    // CSV is plain text with no dedicated magic number, so fall back to a
    // heuristic: if the first 8 bytes contain no NUL (0x00) and are all ASCII,
    // treat the file as CSV-like.
    if buffer.iter().all(|&b| b != 0x00 && b.is_ascii()) {
        return Ok(FileType::Csv);
    }

    Ok(FileType::Unknown)
}

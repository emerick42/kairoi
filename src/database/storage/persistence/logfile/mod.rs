mod encoding;

use std::fs::File;
use std::io::{Error as IoError, ErrorKind, Read, Seek, SeekFrom, Write};
use self::encoding::{Encoder, ParseError, Parser};

pub type Parsed = encoding::Parsed;
pub enum ReadError {
    CorruptedFile,
    UnreadableFile,
}
pub type ReadAllResult = Result<Vec<Parsed>, ReadError>;

/// Read logfile entries from files.
///
/// Logfiles use a simple specific format to be able to store any data. New entries are
/// concatenated to olders. Each entry is an array of bytes, where the first 4 bytes are the size
/// (using the big-endian format) of the data, and the rest is the data.
pub struct Reader<'a> {
    file: &'a mut File,
}

impl<'a> Reader<'a> {
    /// Create a new logfile reader on the given open file.
    pub fn new(file: &'a mut File) -> Self {
        Self { file }
    }

    /// Read all entries from the beginning of the logfile.
    pub fn all(&mut self) -> ReadAllResult {
        self.file.seek(SeekFrom::Start(0))?;

        // Read the whole content of the file incrementally.
        let parser = Parser::new();
        let mut to_parse = Vec::new();
        let mut buffer = [0; 8192];
        let mut results = Vec::new();
        loop {
            let read = match self.file.read(&mut buffer) {
                Ok(read) => read,
                Err(error) if error.kind() == ErrorKind::Interrupted => continue,
                Err(_) => return Err(ReadError::UnreadableFile),
            };
            if read == 0 {
                if to_parse.len() > 0 {
                    return Err(ReadError::CorruptedFile);
                }
                break;
            }

            to_parse.extend(&buffer[0..read]);
            to_parse = match parser.parse(&to_parse) {
                Ok(entries) => {
                    results.extend(entries);

                    Vec::new()
                },
                Err(ParseError::Incomplete(entries, input_left)) => {
                    results.extend(entries);

                    input_left.to_vec()
                },
                Err(ParseError::CorruptedContent) => return Err(ReadError::CorruptedFile),
            };
        };

        return Ok(results);
    }
}

/// Convert all IoError into LoadError::UnreadableFile.
impl From<IoError> for ReadError {
    fn from(error: IoError) -> Self {
        match error {
            _ => Self::UnreadableFile,
        }
    }
}

pub enum WriteError {
    InvalidData,
    WriteFailure,
}
pub type WriteResult = Result<(), WriteError>;

/// Write logfile entries to files.
pub struct Writer<'a> {
    file: &'a mut File,
    encoder: Encoder,
}

impl<'a> Writer<'a> {
    /// Create a new logfile writer on the given open file. The file should be open with write
    /// privileges.
    pub fn new(file: &'a mut File) -> Self {
        Self {
            file,
            encoder: Encoder::new(),
        }
    }

    /// Write the given entry to the logfile at the current cursor position. This method verifies
    /// that data are synchronized to the file system before returning.
    pub fn write_sync(&mut self, entry: &[u8]) -> WriteResult {
        let entry = match self.encoder.encode(entry) {
            Ok(entry) => entry,
            Err(_) => return Err(WriteError::InvalidData),
        };

        match self.file.write_all(&entry) {
            Ok(_) => match self.file.sync_data() {
                Ok(_) => Ok(()),
                Err(_) => {
                    Err(WriteError::WriteFailure)
                },
            },
            Err(_) => {
                Err(WriteError::WriteFailure)
            },
        }
    }

    /// Write the given entry to the logfile at the current cursor position.
    pub fn write(&mut self, entry: &[u8]) -> WriteResult {
        let entry = match self.encoder.encode(entry) {
            Ok(entry) => entry,
            Err(_) => return Err(WriteError::InvalidData),
        };

        match self.file.write_all(&entry) {
            Ok(_) => Ok(()),
            Err(_) => Err(WriteError::WriteFailure),
        }
    }

    /// Synchronize data previously written in this logfile, making sure its content is properly
    /// written on disk. This method should always be used after "write".
    pub fn sync(&mut self) -> WriteResult {
        match self.file.sync_data() {
            Ok(_) => Ok(()),
            Err(_) => Err(WriteError::WriteFailure),
        }
    }
}

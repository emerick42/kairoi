mod parser;

use log::error;
use parser::{Entry as UnderlyingEntry, ParseError, Parser};
use std::fs::File;
use std::fs::OpenOptions;
use std::io::{Error as IoError, ErrorKind, Read, Seek, SeekFrom, Write};

pub enum AppendError {
    InvalidData,
    UnopenableFile,
    WriteFailure,
}
pub type AppendResult = Result<(), AppendError>;
pub type Entry = UnderlyingEntry;
pub enum ResumeError {
    CorruptedFile,
    UnopenableFile,
    UnreadableFile,
}
pub type ResumeResult = Result<Vec<Entry>, ResumeError>;

/// A write-ahead logger (WAL), logging entries to a logfile synchronously.
///
/// The log file use a simple specific format to be able to store any data. New entries are
/// concatenated to olders. Each entry is an array of bytes, where the first 4 bytes are the size
/// (using the big-endian format) of the data, and the rest is the data.
pub struct WriteAheadLogger {
    file: Option<File>,
}

impl WriteAheadLogger {
    /// Create a new WriteAheadLogger.
    pub fn new() -> WriteAheadLogger {
        WriteAheadLogger {
            file: None,
        }
    }

    /// Append the given binary data to the configured logfile synchronously.
    pub fn append(&mut self, data: &[u8]) -> AppendResult {
        let entry = match Self::build_entry(data) {
            Ok(entry) => entry,
            Err(_) => return Err(AppendError::InvalidData),
        };

        let result = match self.open_file() {
            Ok(file) => {
                match file.write_all(&entry) {
                    Ok(_) => match file.sync_data() {
                        Ok(_) => Ok(()),
                        Err(_) => {
                            Err(AppendError::WriteFailure)
                        },
                    },
                    Err(_) => {
                        Err(AppendError::WriteFailure)
                    },
                }
            },
            Err(_) => Err(AppendError::UnopenableFile),
        };

        match result {
            Ok(ok) => Ok(ok),
            Err(error) => {
                self.file = None;

                Err(error)
            },
        }
    }

    /// Resume the write-ahead logging using the current logfile. Retrieve all currently written
    /// entries and position the writing cursor to the end of the file. If an error occurs, the
    /// cursor position is undetermined.
    pub fn resume(&mut self) -> ResumeResult {
        match self.open_file() {
            Ok(file) => {
                file.seek(SeekFrom::Start(0))?;

                // Read the whole content of the file incrementally.
                let parser = Parser::new();
                let mut to_parse = Vec::new();
                let mut buffer = [0; 8192];
                let mut results = Vec::new();
                loop {
                    let read = match file.read(&mut buffer) {
                        Ok(read) => read,
                        Err(error) if error.kind() == ErrorKind::Interrupted => continue,
                        Err(_) => return Err(ResumeError::UnreadableFile),
                    };
                    if read == 0 {
                        if to_parse.len() > 0 {
                            return Err(ResumeError::CorruptedFile);
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
                        Err(ParseError::CorruptedContent) => return Err(ResumeError::CorruptedFile),
                    };
                };

                return Ok(results);
            },
            Err(_) => Err(ResumeError::UnopenableFile),
        }
    }

    /// Open the logfile in append mode if needed (the file may already be opened previously).
    /// Return a handler on the file, or an error if it has been impossible to open it.
    fn open_file(&mut self) -> Result<&mut File, ()> {
        if let None = self.file {
            match OpenOptions::new().read(true).append(true).create(true).open("logfile") {
                Ok(file) => {
                    self.file = Some(file);
                },
                Err(_) => {
                    error!("Unable to open the write-ahead log file.");

                    return Err(())
                },
            }
        };

        match &mut self.file {
            Some(file) => Ok(file),
            None => Err(()),
        }
    }

    /// Build an entry to be appended to the log file. Return an error if the data is too big.
    fn build_entry(data: &[u8]) -> Result<Vec<u8>, ()> {
        match data.len() < u32::MAX as usize && data.len() != 0 {
            true => {
                let mut entry = vec![0; 4 + data.len()];
                &entry[0..4].copy_from_slice(&(data.len() as u32).to_be_bytes());
                &entry[4..].copy_from_slice(data);

                Ok(entry)
            },
            false => Err(()),
        }
    }
}

impl From<IoError> for ResumeError {
    fn from(error: IoError) -> Self {
        match error {
            _ => Self::UnreadableFile,
        }
    }
}

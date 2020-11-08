use log::error;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;

pub enum Error {
    AppendFailed,
}
pub type AppendResult = Result<(), Error>;

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
            Err(_) => return Err(Error::AppendFailed),
        };

        let result = match self.open_file() {
            Ok(file) => {
                match file.write_all(&entry) {
                    Ok(_) => match file.sync_data() {
                        Ok(_) => Ok(()),
                        Err(_) => {
                            Err(Error::AppendFailed)
                        },
                    },
                    Err(_) => {
                        Err(Error::AppendFailed)
                    },
                }
            },
            Err(_) => Err(Error::AppendFailed),
        };

        match result {
            Ok(ok) => Ok(ok),
            Err(error) => {
                self.file = None;

                Err(error)
            },
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
        match data.len() < u32::MAX as usize {
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

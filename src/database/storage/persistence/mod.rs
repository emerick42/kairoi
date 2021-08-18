mod encoder;
mod write_ahead_logger;

use log::{debug};
use self::encoder::{Decoded, Encodable, Encoder};
use self::write_ahead_logger::WriteAheadLogger;

pub type Job = encoder::Job;
pub type JobStatus = encoder::JobStatus;
pub type Rule = encoder::Rule;
pub type Runner = encoder::Runner;
pub enum Entry {
    Job(Job),
    Rule(Rule),
}
pub enum InitializationError {
    InvalidEntry,
    UnreadableFile,
}
pub type InitializationResult = Result<Vec<Entry>, InitializationError>;
pub enum PersistError {
    EncodingFailure,
    WriteAheadLoggerFailure,
}
pub type PersistResult = Result<(), PersistError>;

/// A persisted database storage, handling jobs and rules.
///
/// Internally, it uses a write-ahead logger to quickly and synchronously store data to the file
/// system, so there is no data lost on failure.
pub struct Storage {
    write_ahead_logger: WriteAheadLogger,
    encoder: Encoder,
}

impl Storage {
    /// Create a new Storage.
    pub fn new() -> Storage {
        Storage {
            write_ahead_logger: WriteAheadLogger::new(),
            encoder: Encoder::new(),
        }
    }

    /// Initialize the persistent storage by retrieving data from the file system. On success, it
    /// returns all persisted entries.
    pub fn initialize(&mut self) -> InitializationResult {
        let entries = match self.write_ahead_logger.resume() {
            Ok(entries) => entries,
            Err(_) => return Err(InitializationError::UnreadableFile),
        };

        debug!("{:?} entries have been read from the logfile.", entries.len());

        let mut results = Vec::with_capacity(entries.len());
        for entry in entries {
            match self.encoder.decode(&entry) {
                Ok(decoded) => match decoded {
                    Decoded::Job(job) => results.push(Entry::Job(job)),
                    Decoded::Rule(rule) => results.push(Entry::Rule(rule)),
                },
                Err(_) => return Err(InitializationError::InvalidEntry),
            }
        };

        Ok(results)
    }

    /// Persist the given entry to this storage.
    pub fn persist(&mut self, entry: Entry) -> PersistResult {
        let encoded = match self.encoder.encode(Encodable::from(entry)) {
            Ok(entry) => entry,
            Err(_) => return Err(PersistError::EncodingFailure),
        };

        match self.write_ahead_logger.append(&encoded) {
            Ok(_) => Ok(()),
            Err(_) => Err(PersistError::WriteAheadLoggerFailure),
        }
    }
}

/// Convert entries into encodables.
impl From<Entry> for Encodable {
    fn from(entry: Entry) -> Self {
        match entry {
            Entry::Job(job) => Encodable::Job(job),
            Entry::Rule(rule) => Encodable::Rule(rule),
        }
    }
}

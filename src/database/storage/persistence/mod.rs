//! Database storage persistence, handling jobs and rules.
//!
//! The storage uses an append-only logfile to quickly write new entries on disk. At some points
//! during its lifetime, it may start a background process for compressing the logfile, preventing
//! it from growing infinitely. This compressing process is fully error-proof: the file system is
//! always kept in a recoverable state, in case of system failure.
//!
//! When compressing, the process duplicates a lot of entries in memory, thus increasing the memory
//! usage. The current implementation is optimized for better compressing smaller logfiles, since
//! the entire logfile to compress is loaded in memory. It should, however, be noted that for now,
//! the full content of the compressed file is copied at each compression process, potentially
//! generating high IO pressure following the size of the database.
//!
//! # Internals
//!
//! ## Compression Process
//!
//! When persisting an entry in the persistent storage, a check is made to verify if the logfile
//! contains 5000 or more entries and if there is no compression process running. If it happens, a
//! new compression process is launched. Its first step is to flush the logfile to the file
//! `logfile.to_compress`. This flush happens synchronously with the current persist operation.
//! Once it's done, a background process is started, and the persist operation succeeds. On
//! subsequent persist operations, a check is made to verify that the background compression
//! process has been terminated.
//!
//! The background process manipulates the 3 following files:
//!
//! * `logfile.to_compress`, containing the content of the flushed logfile,
//! * `logfile.compressed`, containing entries compressed during previous compressions,
//! * and `logfile.compressing`, a temporary file, filled by the compression process, that will
//!   serve as the new compressed file.
//!
//! The whole content of `logfile.to_compress` and `logfile.compressed` is currently loaded in
//! memory. While this is a deliberate choice for the first file, the second file could completely
//! be read as a stream (it's an improvement for later).
//!
//! The background process starts by reading entries from `logfile.to_compress` and apply a
//! deduplication on them (keeping only the latest entry for a given item). It then goes through
//! all entries from `logfile.compressed`, and insert entries being absent from
//! `logfile.to_compress` into `logfile.compressing`. After that, it inserts all deduplicated
//! entries from `logfile.to_compress` into `logfile.compressing`. Finally, it moves
//! `logfile.compressing` to replace `logfile.compressed`, deletes `logfile.to_compress`, and
//! notifies the main process that everything went well.
//!
//! At any time, the background process can fail, keeping `logfile.compressed` and
//! `logfile.to_compress` in their original state. It can then be resumed without any data loss.
//! The only loss that can occur happens if we flush the logfile again during a compression process
//! (it would overwrite the content of `logfile.to_compress` with the new content of `logfile`,
//! thus losing it). To prevent this, the initialization process is improved to load the content of
//! `logfile.to_compress` and resume compressing if needed. It should be noted that, in order for
//! this process to work, all 3 logfiles must be on the same file system, supporting the atomic
//! file move.

mod background;
mod encoder;
mod logfile;

use background::{Process, Status, TaskError, TaskResult};
use log::{debug, error};
use self::encoder::{Decoded, Encodable, Encoder};
use std::collections::HashMap;
use std::fs::{File, OpenOptions, remove_file, rename};
use std::io::ErrorKind;

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
    WriteFailure,
}
pub type PersistResult = Result<(), PersistError>;

const COMPRESSION_THRESHOLD: usize = 5000;

pub struct Storage {
    encoder: Encoder,
    file: Option<File>,
    process: Option<Process>,
    logfile_size: usize,
}

impl Storage {
    /// Create a new Storage.
    pub fn new() -> Storage {
        Storage {
            encoder: Encoder::new(),
            file: None,
            process: None,
            logfile_size: 0,
        }
    }

    /// Initialize the persistent storage by retrieving data from the file system. On success, it
    /// returns the last log entry for each persisted item. This method should only be used once,
    /// and before any [`persist`] call.
    pub fn initialize(&mut self) -> InitializationResult {
        let mut entries = Vec::new();

        // Load entries from "logfile.compressed", if it exists.
        match OpenOptions::new().read(true).open("logfile.compressed") {
            Ok(mut file) => {
                let mut reader = logfile::Reader::new(&mut file);
                let loaded = match reader.all() {
                    Ok(entries) => entries,
                    Err(_) => return Err(InitializationError::UnreadableFile),
                };
                debug!("{:?} entries have been read from 'logfile.compressed'.", loaded.len());
                entries.extend(loaded);
            },
            Err(error) if (error.kind() == ErrorKind::NotFound) => (),
            Err(_) => return Err(InitializationError::UnreadableFile),
        };

        // If "logfile.to_compress" exists, load entries from it and resume compressing.
        match OpenOptions::new().read(true).open("logfile.to_compress") {
            Ok(mut file) => {
                let mut reader = logfile::Reader::new(&mut file);
                let loaded = match reader.all() {
                    Ok(entries) => entries,
                    Err(_) => return Err(InitializationError::UnreadableFile),
                };
                debug!("{:?} entries have been read from 'logfile.to_compress'.", loaded.len());
                entries.extend(loaded);
                drop(file);

                debug!("Resuming the compression process.");
                self.process = Some(Process::execute(Self::compress));
            },
            Err(error) if (error.kind() == ErrorKind::NotFound) => (),
            Err(_) => return Err(InitializationError::UnreadableFile),
        };

        // Load entries from "logfile".
        match OpenOptions::new().read(true).open("logfile") {
            Ok(mut file) => {
                let mut reader = logfile::Reader::new(&mut file);
                let loaded = match reader.all() {
                    Ok(entries) => entries,
                    Err(_) => return Err(InitializationError::UnreadableFile),
                };
                debug!("{:?} entries have been read from 'logfile'.", loaded.len());
                self.logfile_size = loaded.len();
                entries.extend(loaded);
            },
            Err(error) if (error.kind() == ErrorKind::NotFound) => (),
            Err(_) => return Err(InitializationError::UnreadableFile),
        };

        debug!("Starting to decode all {:?} entries read.", entries.len());
        let mut unique_results = HashMap::new();
        for entry in entries {
            match self.encoder.decode(&entry) {
                Ok(decoded) => unique_results.insert(decoded.get_subject(), Entry::from(decoded)),
                Err(_) => return Err(InitializationError::InvalidEntry),
            };
        };
        let mut results = Vec::with_capacity(unique_results.len());
        for entry in unique_results.into_values() {
            results.push(entry);
        };

        Ok(results)
    }

    /// Persist the given entry to this storage. When needed, it may start the background process
    /// for compressing the logfile.
    pub fn persist(&mut self, entry: Entry) -> PersistResult {
        let encoded = match self.encoder.encode(Encodable::from(entry)) {
            Ok(entry) => entry,
            Err(_) => return Err(PersistError::EncodingFailure),
        };

        // I'm not sure how to borrow this mutable reference on file properly. It should exist
        // since we create it, but still there is a second match here. It may be improved.
        if let None = self.file {
            self.file = match OpenOptions::new().append(true).create(true).open("logfile") {
                Ok(file) => Some(file),
                Err(_) => return Err(PersistError::WriteFailure),
            };
        };
        let file = match &mut self.file {
            Some(file) => file,
            None => return Err(PersistError::WriteFailure),
        };

        let mut writer = logfile::Writer::new(file);

        if let Err(_) = writer.write_sync(&encoded) {
            return Err(PersistError::WriteFailure);
        };

        self.logfile_size += 1;

        // Check the status of the last compression process, to finalize it for this storage if
        // needed.
        if let Some(process) = &self.process {
            match process.status() {
                Status::Success => {
                    debug!("The compression process has terminated with a success.");
                    self.process = None;
                },
                Status::Failure(_) => {
                    debug!("The compression process has terminated with a failure.");
                    self.process = None;
                },
                Status::Running => {},
                Status::Lost => {
                    error!("Resuming the compression process after its loss.");
                    self.process = Some(Process::execute(Self::compress));
                }
            };
        };

        // Check if compression is needed and launch a background compression.
        let already_compressing = match &self.process {
            Some(_) => true,
            None => false,
        };
        let should_start_compression = self.logfile_size >= COMPRESSION_THRESHOLD;
        if !already_compressing && should_start_compression {
            debug!("Starting the compression process.");

            self.file = None;
            debug!("Moving 'logfile' to 'logfile.to_compress'.");
            if let Err(_) = rename("logfile", "logfile.to_compress") {
                error!("Unable to move 'logfile' to 'logfile.to_compress'.");

                return Ok(());
            }
            self.logfile_size = 0;

            self.process = Some(Process::execute(Self::compress));
        };

        Ok(())
    }

    /// Compress "logfile.compressed" and "logfile.to_compress" into "logfile.compressed". This
    /// function is used as a task of a background process.
    fn compress() -> TaskResult {
        let encoder = Encoder::new();

        let mut to_compress_file = match OpenOptions::new().read(true).open("logfile.to_compress") {
            Ok(file) => file,
            Err(_) => {
                error!("Unable to open 'logfile.to_compress'.");

                return Err(TaskError::Failure);
            },
        };
        let mut compressed_file = match OpenOptions::new().read(true).write(true).create(true).open("logfile.compressed") {
            Ok(file) => file,
            Err(_) => {
                error!("Unable to open 'logfile.compressed'.");

                return Err(TaskError::Failure);
            },
        };
        let mut compressing_file = match OpenOptions::new().write(true).create(true).truncate(true).open("logfile.compressing") {
            Ok(file) => file,
            Err(_) => {
                error!("Unable to open 'logfile.compressing'.");

                return Err(TaskError::Failure);
            },
        };

        // Read all "logfile.to_compress" entries, deduplicate them and store them in memory.
        // Entries are stored indexed by their subject.
        let mut to_compress_reader = logfile::Reader::new(&mut to_compress_file);
        debug!("Starting to read entries from 'logfile.to_compress'.");
        let to_compress_entries = match to_compress_reader.all() {
            Ok(entries) => entries,
            Err(_) => return Err(TaskError::Failure),
        };
        let mut to_compress_decoded = Vec::with_capacity(to_compress_entries.len());
        for entry in &to_compress_entries {
            match encoder.decode(&entry) {
                Ok(entry) => to_compress_decoded.push(entry),
                Err(_) => return Err(TaskError::Failure),
            };
        };
        let mut to_compress = HashMap::new();
        debug!("Starting to deduplicate entries from 'logfile.to_compress'.");
        for (index, entry) in to_compress_entries.iter().enumerate().rev() {
            let decoded = &to_compress_decoded[index];
            let subject = decoded.get_subject();
            if !to_compress.contains_key(&subject) {
                to_compress.insert(subject, entry.clone());
            };
        }

        // @TODO: Read compressed entries with iterations, instead of loading everything in memory.
        let mut compressed_reader = logfile::Reader::new(&mut compressed_file);
        debug!("Starting to read entries from 'logfile.compressed'.");
        let compressed_entries = match compressed_reader.all() {
            Ok(entries) => entries,
            Err(_) => return Err(TaskError::Failure),
        };

        let mut compressing_writer = logfile::Writer::new(&mut compressing_file);

        // For each entry from "logfile.compressed", check if there is a more recent entry in
        // "logfile.to_compress". If it's not the case, add the entry to "logfile.compressing".
        // Since "logfile.compressed" only contains unique entries, there is no need to double
        // check in the file itself.
        debug!("Starting to write entries from 'logfile.compressed' to 'logfile.compressing'.");
        for compressed_entry in &compressed_entries {
            let compressed_decoded = match encoder.decode(compressed_entry) {
                Ok(entry) => entry,
                Err(_) => return Err(TaskError::Failure),
            };
            if !to_compress.contains_key(&compressed_decoded.get_subject()) {
                if let Err(_) = compressing_writer.write(compressed_entry) {
                    return Err(TaskError::Failure);
                };
            }
        };

        debug!("Starting to write entries from 'logfile.to_compress' to 'logfile.compressing'.");
        for entry in to_compress.values() {
            if let Err(_) = compressing_writer.write(entry) {
                return Err(TaskError::Failure);
            };
        }

        // Synchronize the written file to the file system, making sure the operation is fully
        // completed.
        if let Err(_) = compressing_writer.sync() {
            return Err(TaskError::Failure);
        }

        // Close "logfile.to_compress", "logfile.compressing" and "logfile.compressed", then
        // complete the compression process by replacing the old compressed file by the new one and
        // removing "logfile.to_compress". These two operations can be done un-atomically, because
        // a failure between them will only cause the compression to be re-started, not corrupting
        // any data.
        debug!("Replacing 'logfile.compressed' by 'logfile.compressing'.");
        if let Err(_) = rename("logfile.compressing", "logfile.compressed") {
            error!("Unable to rename 'logfile.compressing' to 'logfile.compressed'.");

            return Err(TaskError::Failure);
        }
        debug!("Removing 'logfile.to_compress'.");
        if let Err(_) = remove_file("logfile.to_compress") {
            error!("Unable to remove 'logfile.to_compress'.");

            return Err(TaskError::Failure);
        };

        return Ok(());
    }
}

/// Convert Entry into Encodable.
impl From<Entry> for Encodable {
    fn from(entry: Entry) -> Self {
        match entry {
            Entry::Job(job) => Encodable::Job(job),
            Entry::Rule(rule) => Encodable::Rule(rule),
        }
    }
}

// Convert Decoded into Entry.
impl From<Decoded> for Entry {
    fn from(decoded: Decoded) -> Self {
        match decoded {
            Decoded::Job(job) => Entry::Job(job),
            Decoded::Rule(rule) => Entry::Rule(rule),
        }
    }
}

impl Decoded {
    /// Get the subject of this decoded entry. The subject is a unique representation of an item
    /// concerned by a log entry. For example, two entries about the same job "job.1" will have the
    /// same subject, but an entry about a job "job.1" will not have the same subject than an entry
    /// about the job "job.2". An entry about a job "app.1" will also not have the same subject
    /// than an entry about the rule "app.1".
    fn get_subject(&self) -> String {
        let (prefix, identifier) = match self {
            Decoded::Job(job) => ('j', &job.identifier),
            Decoded::Rule(rule) => ('r', &rule.identifier),
        };

        let mut subject = String::with_capacity(identifier.capacity() + 1);

        subject.push(prefix);
        subject.push_str(&identifier);

        subject
    }
}

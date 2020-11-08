use crate::execution::runner::Runner;
use super::{Job, JobStatus};
use super::Rule;

pub enum Encodable<'a> {
    Job(&'a Job),
    Rule(&'a Rule),
}
pub type EncodeResult = Result<Vec<u8>, ()>;

/// Encode values that must be stored in a file as decodable binary.
pub struct Encoder {}

impl Encoder {
    /// Create a new encoder.
    pub fn new() -> Encoder {
        Encoder {}
    }

    /// Encode the given encodable value into an array of bytes. See specialized encode methods for
    /// more details on how each type of value is encoded.
    pub fn encode(&self, value: Encodable) -> EncodeResult {
        match value {
            Encodable::Job(job) => self.encode_job(job),
            Encodable::Rule(rule) => self.encode_rule(rule),
        }
    }

    /// Encode the given job into an array of bytes.
    ///
    /// A job is encoded concatenating the following arrays of bytes:
    /// - [u8: 1]: the type of this value (0 for jobs),
    /// - [u8: 2]: the size of the job's identifier string as big-endian,
    /// - [u8: identifier_size]: the identifier of the job,
    /// - [u8: 8]: the execution timestamp of the job (with nanoseconds precision) as big-endian,
    /// - [u8: 1]: the status of the job (0 = planned, 1 = triggered, 2 = executed, 3 = failed).
    fn encode_job(&self, job: &Job) -> EncodeResult {
        let identifier_size = match job.get_identifier().len() > u16::MAX as usize {
            true => return Err(()),
            false => job.get_identifier().len() as u16,
        };

        let mut result = vec![0; 12 + identifier_size as usize];
        result[0] = 0;
        &result[1..3].copy_from_slice(&identifier_size.to_be_bytes());
        &result[3..(3 + identifier_size as usize)].copy_from_slice(job.get_identifier().as_bytes());
        &result[(3 + identifier_size as usize)..(11 + identifier_size as usize)].copy_from_slice(&job.get_execution().timestamp_nanos().to_be_bytes());
        result[11 + identifier_size as usize] = match job.get_status() {
            JobStatus::Planned => 0,
            JobStatus::Triggered => 1,
            JobStatus::Executed => 2,
            JobStatus::Failed => 3,
        };

        Ok(result)
    }

    /// Encode the given rule into an array of bytes.
    ///
    /// A rule is encoded concatenating the following arrays of bytes:
    /// - [u8: 1]: the type of this value (1 for rules),
    /// - [u8: 2]: the size of the rule's identifier string as big-endian,
    /// - [u8: identifier_size]: the identifier of the rule,
    /// - [u8: 2]: the size of the rule's pattern string as big-endian,
    /// - [u8: pattern_size]: the pattern of the rule,
    /// - [u8: 1]: the runner's type of the rule (0 = shell, 1 = amqp),
    /// - [u8: various_size]: the runner configuration, depending on its type.
    fn encode_rule(&self, rule: &Rule) -> EncodeResult {
        let identifier_size = match rule.get_identifier().len() > u16::MAX as usize {
            true => return Err(()),
            false => rule.get_identifier().len() as u16,
        };
        let pattern_size = match rule.get_pattern().len() > u16::MAX as usize {
            true => return Err(()),
            false => rule.get_pattern().len() as u16,
        };

        // Encode the runner configuration.
        #[allow(unreachable_patterns)]
        let encoded_runner = match rule.get_runner() {
            #[cfg(feature = "runner-shell")]
            Runner::Shell {command} => {
                let command_size = match command.len() > u16::MAX as usize {
                    true => return Err(()),
                    false => command.len() as u16,
                };
                let mut result = vec![0; 3 + command_size as usize];
                result[0] = 0;
                &result[1..3].copy_from_slice(&command_size.to_be_bytes());
                &result[3..].copy_from_slice(command.as_bytes());

                result
            },
            #[cfg(feature = "runner-amqp")]
            Runner::Amqp {dsn, exchange, routing_key} => {
                let dsn_size = match dsn.len() > u16::MAX as usize {
                    true => return Err(()),
                    false => dsn.len() as u16,
                };
                let exchange_size = match exchange.len() > u16::MAX as usize {
                    true => return Err(()),
                    false => exchange.len() as u16,
                };
                let routing_key_size = match routing_key.len() > u16::MAX as usize {
                    true => return Err(()),
                    false => routing_key.len() as u16,
                };
                let mut result = vec![0; 7 + dsn_size as usize + exchange_size as usize + routing_key_size as usize];
                result[0] = 1;
                &result[1..3].copy_from_slice(&dsn_size.to_be_bytes());
                &result[3..(3 + dsn_size as usize)].copy_from_slice(dsn.as_bytes());
                &result[(3 + dsn_size as usize)..(5 + dsn_size as usize)].copy_from_slice(&exchange_size.to_be_bytes());
                &result[(5 + dsn_size as usize)..(5 + dsn_size as usize + exchange_size as usize)].copy_from_slice(exchange.as_bytes());
                &result[(5 + dsn_size as usize + exchange_size as usize)..(7 + dsn_size as usize + exchange_size as usize)].copy_from_slice(&routing_key_size.to_be_bytes());
                &result[(7 + dsn_size as usize + exchange_size as usize)..].copy_from_slice(routing_key.as_bytes());

                result
            },
            _ => return Err(()),
        };

        // Encode the rule.
        let mut result = vec![0; 5 + identifier_size as usize + pattern_size as usize + encoded_runner.len()];
        result[0] = 1;
        &result[1..3].copy_from_slice(&identifier_size.to_be_bytes());
        &result[3..(3 + identifier_size as usize)].copy_from_slice(rule.get_identifier().as_bytes());
        &result[(3 + identifier_size as usize)..(5 + identifier_size as usize)].copy_from_slice(&pattern_size.to_be_bytes());
        &result[(5 + identifier_size as usize)..(5 + identifier_size as usize + pattern_size as usize)].copy_from_slice(rule.get_pattern().as_bytes());
        &result[(5 + identifier_size as usize + pattern_size as usize)..].copy_from_slice(&encoded_runner);

        Ok(result)
    }
}

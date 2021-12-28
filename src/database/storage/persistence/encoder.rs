use chrono::DateTime;
use chrono::offset::{TimeZone, Utc};
use nom::branch::alt;
use nom::bytes::complete::{tag, take};
use nom::combinator::{all_consuming, flat_map};
use nom::Err as NomErr;
use nom::error::{Error, ErrorKind};
use nom::IResult;
use nom::number::complete::{be_i64, be_u16, be_u8};
use nom::sequence::tuple;

#[cfg_attr(test, derive(Debug, PartialEq))]
pub enum JobStatus {
    Planned,
    Triggered,
    Executed,
    Failed,
}
/// Jobs to be encoded and decoded.
#[cfg_attr(test, derive(Debug, PartialEq))]
pub struct Job {
    pub identifier: String,
    pub execution: DateTime<Utc>,
    pub status: JobStatus,
}
#[cfg_attr(test, derive(Debug, PartialEq))]
pub enum Runner {
    Amqp {
        dsn: String,
        exchange: String,
        routing_key: String,
    },
    Shell {
        command: String,
    },
}
/// Rules to be encoded and decoded.
#[cfg_attr(test, derive(Debug, PartialEq))]
pub struct Rule {
    pub identifier: String,
    pub pattern: String,
    pub runner: Runner,
}
#[cfg_attr(test, derive(Debug, PartialEq))]
pub enum Decoded {
    Job(Job),
    Rule(Rule),
}
#[cfg_attr(test, derive(Debug, PartialEq))]
pub enum DecodeError {
    InvalidData,
}
pub type DecodeResult = Result<Decoded, DecodeError>;
pub enum Encodable {
    Job(Job),
    Rule(Rule),
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
            Encodable::Job(job) => self.encode_job(&job),
            Encodable::Rule(rule) => self.encode_rule(&rule),
        }
    }

    /// Decode the given data into a Decoded enum. See specialized encode methods for more details
    /// on how each type of value is encoded.
    pub fn decode<'a>(&self, data: &'a [u8]) -> DecodeResult {
        // Handle job entries.
        let job = |input: &'a [u8]| -> IResult<&'a [u8], Decoded> {
            let entry_type_job = tag([0]);
            let job_identifier = sized_utf8_string();
            let job_timestamp = |input: &'a [u8]| -> IResult<&'a [u8], DateTime<Utc>> {
                let (input, timestamp) = be_i64(input)?;

                Ok((input, Utc.timestamp(timestamp / 1_000_000_000, (timestamp % 1_000_000_000) as u32)))
            };
            let job_status = |input: &'a [u8]| -> IResult<&'a [u8], JobStatus> {
                let (input_left, status) = be_u8(input)?;
                let status = match status {
                    0 => JobStatus::Planned,
                    1 => JobStatus::Triggered,
                    2 => JobStatus::Executed,
                    3 => JobStatus::Failed,
                    _ => return Err(NomErr::Error(Error { input, code: ErrorKind::Tag })),
                };

                Ok((input_left, status))
            };
            let (input, (_, identifier, execution, status)) = tuple((entry_type_job, job_identifier, job_timestamp, job_status))(input)?;

            Ok((input, Decoded::Job(Job { identifier, execution, status })))
        };

        // Handle rule entries.
        let rule = |input: &'a [u8]| -> IResult<&'a [u8], Decoded> {
            let entry_type_rule = tag([1]);
            let rule_identifier = sized_utf8_string();
            let rule_pattern = sized_utf8_string();
            let rule_runner = |input: &'a [u8]| -> IResult<&'a [u8], Runner> {
                let (input, runner_type) = be_u8(input)?;

                match runner_type {
                    0 => {
                        let command = sized_utf8_string();

                        let (input, command) = command(input)?;

                        Ok((input, Runner::Shell { command: command }))
                    },
                    1 => {
                        let dsn = sized_utf8_string();
                        let exchange = sized_utf8_string();
                        let routing_key = sized_utf8_string();

                        let (input, (dsn, exchange, routing_key)) = tuple((dsn, exchange, routing_key))(input)?;

                        Ok((input, Runner::Amqp { dsn: dsn, exchange: exchange, routing_key: routing_key }))
                    },
                    _ => return Err(NomErr::Failure(Error { input, code: ErrorKind::Tag })),
                }
            };
            let (input, (_, identifier, pattern, runner)) = tuple((entry_type_rule, rule_identifier, rule_pattern, rule_runner))(input)?;

            Ok((input, Decoded::Rule(Rule { identifier, pattern, runner })))
        };

        match all_consuming(alt((job, rule)))(data) {
            Ok((_, decoded)) => Ok(decoded),
            Err(_) => Err(DecodeError::InvalidData)
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
        let identifier_size = match job.identifier.len() > u16::MAX as usize {
            true => return Err(()),
            false => job.identifier.len() as u16,
        };

        let mut result = vec![0; 12 + identifier_size as usize];
        result[0] = 0;
        result[1..3].copy_from_slice(&identifier_size.to_be_bytes());
        result[3..(3 + identifier_size as usize)].copy_from_slice(job.identifier.as_bytes());
        result[(3 + identifier_size as usize)..(11 + identifier_size as usize)].copy_from_slice(&job.execution.timestamp_nanos().to_be_bytes());
        result[11 + identifier_size as usize] = match job.status {
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
        let identifier_size = match rule.identifier.len() > u16::MAX as usize {
            true => return Err(()),
            false => rule.identifier.len() as u16,
        };
        let pattern_size = match rule.pattern.len() > u16::MAX as usize {
            true => return Err(()),
            false => rule.pattern.len() as u16,
        };

        // Encode the runner configuration.
        let encoded_runner = match &rule.runner {
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
                result[1..3].copy_from_slice(&dsn_size.to_be_bytes());
                result[3..(3 + dsn_size as usize)].copy_from_slice(dsn.as_bytes());
                result[(3 + dsn_size as usize)..(5 + dsn_size as usize)].copy_from_slice(&exchange_size.to_be_bytes());
                result[(5 + dsn_size as usize)..(5 + dsn_size as usize + exchange_size as usize)].copy_from_slice(exchange.as_bytes());
                result[(5 + dsn_size as usize + exchange_size as usize)..(7 + dsn_size as usize + exchange_size as usize)].copy_from_slice(&routing_key_size.to_be_bytes());
                result[(7 + dsn_size as usize + exchange_size as usize)..].copy_from_slice(routing_key.as_bytes());

                result
            },
            Runner::Shell {command} => {
                let command_size = match command.len() > u16::MAX as usize {
                    true => return Err(()),
                    false => command.len() as u16,
                };
                let mut result = vec![0; 3 + command_size as usize];
                result[0] = 0;
                result[1..3].copy_from_slice(&command_size.to_be_bytes());
                result[3..].copy_from_slice(command.as_bytes());

                result
            },
        };

        // Encode the rule.
        let mut result = vec![0; 5 + identifier_size as usize + pattern_size as usize + encoded_runner.len()];
        result[0] = 1;
        result[1..3].copy_from_slice(&identifier_size.to_be_bytes());
        result[3..(3 + identifier_size as usize)].copy_from_slice(rule.identifier.as_bytes());
        result[(3 + identifier_size as usize)..(5 + identifier_size as usize)].copy_from_slice(&pattern_size.to_be_bytes());
        result[(5 + identifier_size as usize)..(5 + identifier_size as usize + pattern_size as usize)].copy_from_slice(rule.pattern.as_bytes());
        result[(5 + identifier_size as usize + pattern_size as usize)..].copy_from_slice(&encoded_runner);

        Ok(result)
    }
}

/// A Nom parser, to parse valid UTF-8 strings prefixed by their size (in big-endian, on 16 bits).
fn sized_utf8_string() -> impl Fn(&[u8]) -> IResult<&[u8], String> {
    move |input: &[u8]| {
        let size = be_u16;
        let (input, string) = flat_map(size, take)(input)?;

        let string = match String::from_utf8(string.to_vec()) {
            Ok(string) => string,
            Err(_) => return Err(NomErr::Failure(Error { input, code: ErrorKind::Tag })),
        };

        Ok((input, string))
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use chrono::offset::Utc;

    #[test]
    fn test_encode() {
        let encoder = Encoder::new();

        // Test job encoding.
        assert_eq!(
            encoder.encode(Encodable::Job(Job { identifier: String::from("toto"), execution: Utc.ymd(2020, 11, 15).and_hms(16, 30, 00), status: JobStatus::Planned })),
            Ok(vec![0, 0, 4, 116, 111, 116, 111, 22, 71, 187, 92, 238, 225, 80, 0, 0]),
        );
        assert_eq!(
            encoder.encode(Encodable::Job(Job { identifier: String::from("tatat"), execution: Utc.ymd(2020, 11, 15).and_hms(16, 30, 00), status: JobStatus::Executed })),
            Ok(vec![0, 0, 5, 116, 97, 116, 97, 116, 22, 71, 187, 92, 238, 225, 80, 0, 2]),
        );
        assert_eq!(
            encoder.encode(Encodable::Rule(Rule { identifier: String::from("t"), pattern: String::from("toto"), runner: Runner::Shell { command: String::from("titi") }})),
            Ok(vec![1, 0, 1, 116, 0, 4, 116, 111, 116, 111, 0, 0, 4, 116, 105, 116, 105]),
        );
        assert_eq!(
            encoder.encode(Encodable::Rule(Rule { identifier: String::from("ta"), pattern: String::from("tot"), runner: Runner::Amqp { dsn: String::from("titit"), exchange: String::from(""), routing_key: String::from("a") }})),
            Ok(vec![1, 0, 2, 116, 97, 0, 3, 116, 111, 116, 1, 0, 5, 116, 105, 116, 105, 116, 0, 0, 0, 1, 97]),
        );
    }

    #[test]
    fn test_decode() {
        let encoder = Encoder::new();

        // Test basic valid buffers.
        assert_eq!(
            encoder.decode(&vec![0, 0, 4, 116, 111, 116, 111, 22, 71, 187, 92, 238, 225, 80, 0, 0]),
            Ok(Decoded::Job(Job { identifier: String::from("toto"), execution: Utc.ymd(2020, 11, 15).and_hms(16, 30, 00), status: JobStatus::Planned })),
        );
        assert_eq!(
            encoder.decode(&vec![0, 0, 5, 116, 97, 116, 97, 116, 22, 71, 187, 92, 238, 225, 80, 0, 2]),
            Ok(Decoded::Job(Job { identifier: String::from("tatat"), execution: Utc.ymd(2020, 11, 15).and_hms(16, 30, 00), status: JobStatus::Executed })),
        );
        assert_eq!(
            encoder.decode(&vec![1, 0, 1, 116, 0, 4, 116, 111, 116, 111, 0, 0, 4, 116, 105, 116, 105]),
            Ok(Decoded::Rule(Rule { identifier: String::from("t"), pattern: String::from("toto"), runner: Runner::Shell { command: String::from("titi") }})),
        );
        assert_eq!(
            encoder.decode(&vec![1, 0, 2, 116, 97, 0, 3, 116, 111, 116, 1, 0, 5, 116, 105, 116, 105, 116, 0, 0, 0, 1, 97]),
            Ok(Decoded::Rule(Rule { identifier: String::from("ta"), pattern: String::from("tot"), runner: Runner::Amqp { dsn: String::from("titit"), exchange: String::from(""), routing_key: String::from("a") }})),
        );
        // Test invalid entries.
        assert_eq!(
            encoder.decode(&vec![]),
            Err(DecodeError::InvalidData),
        );
        assert_eq!(
            encoder.decode(&vec![0, 0, 4]),
            Err(DecodeError::InvalidData),
        );
        assert_eq!(
            encoder.decode(&vec![0, 0, 4, 116, 111, 116, 111, 22, 71, 187, 92, 238, 225, 80, 0, 0, 255]),
            Err(DecodeError::InvalidData),
        );
    }
}

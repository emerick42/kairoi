# Kairoi Official Documentation

This file is the root of the Kairoi Official Documentation.

## Quick Words

Kairoi is a **Time-based Job Scheduler**. It works as a server allowing its clients to schedule jobs to be executed in the future, using a simple text protocol (read more about the protocol in the [Kairoi Client Protocol documentation](client-protocol.md)).

Once the job execution time is past, Kairoi automatically triggers a job execution on a matching configured runner (read more about runners in the [Kairoi Runners documentation](runners.md)). In its default configuration, Kairoi guarantees [ACID](https://en.wikipedia.org/wiki/ACID) properties on its transactions. Kairoi also uses a _at-least once_ delivery model: each job is guaranteed to be processed at-least once, at some point after its execution date, but can also be processed more than one time. Thus, domain code handling jobs should be [idempotent](https://en.wikipedia.org/wiki/Idempotence).

### Summary

- [Kairoi Client Protocol Documentation](client-protocol.md)
- [Kairoi Instructions Reference](instructions.md)
- [Kairoi Runners Documentation](runners.md)
- [Kairoi Server Configuration Reference](configuration.md)

## Usage

## Internals

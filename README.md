# Kairoi

**Kairoi** is a _Dynamic_, _Accurate_ and _Scalable_ **Time-based Job Scheduler** written in Rust.

## Quick Words

Kairoi is a **Time-based Job Scheduler**. It works as a server allowing its clients to schedule jobs to be executed in the future, using a simple text protocol (read more about the protocol in the [Kairoi Client Protocol documentation](docs/client-protocol.md)).

Once the job execution time is past, Kairoi automatically triggers a job execution on a matching configured runner (read more about runners in the [Kairoi Runners documentation](docs/runners.md)).

Kairoi currently targets running on Linux operating systems.

## Documentation

* [Kairoi Official Documentation](docs/index.md)

## Development

Developping on Kairoi requires you to have [Rust installed on your machine](https://www.rust-lang.org/tools/install) in its latest version (`1.44.0` as of today).

### Installation

You can build the development version of Kairoi by cloning this repository, then using :

```
$> cargo build
```

at the root of this repository. It will automatically download and install all required dependencies.

# Kairoi Official Documentation

This file is the root of the Kairoi Official Documentation.

## Quick Words

Kairoi is a **Time-based Job Scheduler**. It works as a server allowing its clients to schedule jobs to be executed in the future, using a simple text protocol (read more about the protocol in the [Kairoi Client Protocol documentation](client-protocol.md)).

Once the job execution time is past, Kairoi automatically triggers a job execution on a matching configured runner (read more about runners in the [Kairoi Runners documentation](runners.md)). In its default configuration, Kairoi guarantees [ACID](https://en.wikipedia.org/wiki/ACID) properties on its transactions. Kairoi also uses a _at-least once_ delivery model: each job is guaranteed to be processed at-least once, at some point after its execution date, but can also be processed more than one time. Thus, domain code handling jobs should be [idempotent](https://en.wikipedia.org/wiki/Idempotence).

A Kairoi server can be started by using the binary compiled from sources. For example, a new instance with default configurations can be started from any directory with the following shell command:

```sh
kairoi
```

This server will start, initializing with no data, and listening to clients on `127.0.0.1:5678`. A server should typically be run in background (using a process control system, such as [systemd](https://systemd.io/)).

### Summary

- [Kairoi Client Protocol Documentation](client-protocol.md)
- [Kairoi Instructions Reference](instructions.md)
- [Kairoi Runners Documentation](runners.md)
- [Kairoi Server Configuration Reference](configuration.md)
- [Kairoi Compilation Documentation](compilation.md)

## Usage

While the main user-defined configuration of a Kairoi server comes from the `configuration.toml` file (see the [Kairoi Server Configuration Reference](configuration.md)), the server binary provides a few handy arguments.

### Help

`-h`, `--help`

It displays the help message, containing the list of available options and arguments.

```sh
kairoi -h
```

### Version

`-V`, `--version`

It displays the version of the compiled binary in use.

```sh
kairoi -V
```

### Config

`-c`, `--config` `<FILE>`

It sets the path of the configuration file to use, for the current execution only. The path can be either absolute or relative. When relative, it is computed from the executable launch directory. This argument overwrites the configuration path that can be set at compilation time (being the relative path `configuration.toml` by default).

```sh
kairoi -c var/configuration.toml
kairoi --config=/etc/kairoi/configuration.toml
```

## Internals

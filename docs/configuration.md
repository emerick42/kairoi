# Kairoi Server Configuration

## Quick Words

Kairoi servers can be configured at runtime using a [TOML](https://toml.io/en/) configuration file. The configuration file must be located at `configuration.toml`, relative to the executable directory. Since every option has a default value, a Kairoi server can be started without any configuration file.

Here is an example of configuration file with all default options explicitly set:

```toml
[log]
level = "info" # One of "trace", "debug", "info", "warn", "error" or "off".
```

## Usage

### Log

The `log` table contains all configuration options related to Kairoi's logging. Currently, logging can only be configured to filter messages based on their log level.

#### Level

`log.level`: `String` (default: `info`)

This option can have a value being either `trace`, `debug`, `info`, `warn`, `error` or `off`. The `trace` log level will output every log messages, and `off` log level will output no message at all.

## Internals

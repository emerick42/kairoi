# Kairoi Server Configuration

## Quick Words

Kairoi servers can be configured at runtime using a [TOML](https://toml.io/en/) configuration file. The configuration file must be located at `configuration.toml`, relative to the executable directory. Since every option has a default value, a Kairoi server can be started without any configuration file.

Here is an example of configuration file with all default options explicitly set:

```toml
[log]
level = "info" # One of "trace", "debug", "info", "warn", "error" or "off".

[controller]
listen = "127.0.0.1:5678" # You can use "0.0.0.0:5678" to accept connections from any client.
```

## Usage

### Log

The `log` table contains all configuration options related to Kairoi's logging. Currently, logging can only be configured to filter messages based on their log level.

#### Level

`log.level`: `String` (default: `info`)

This option can have a value being either `trace`, `debug`, `info`, `warn`, `error` or `off`. The `trace` log level will output every log messages, and `off` log level will output no message at all.

### Controller

The `controller` table contains all configuration options related to Kairoi's controller, the component responsible for handling clients.

#### Listen

`controller.listen`: `String` (default: `127.0.0.1:5678`)

This option configures the address on which the controller listens to clients. It can be used to restrict access to certain clients. By default, it uses the most restrictive `127.0.0.1:5678`, accepting only connections from localhost clients. It can be set to `0.0.0.0:5678` to accept any client. The port can also be set to `127.0.0.1:0` to request that the OS assigns a port to the listener (although currently, the assigned port is only retrievable from `info` logs in a human readable format).

## Internals

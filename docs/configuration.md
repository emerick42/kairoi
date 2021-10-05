# Kairoi Server Configuration

## Quick Words

Kairoi servers can be configured at runtime using a [TOML](https://toml.io/en/) configuration file. The configuration file must be located at `configuration.toml`, relative to the executable directory. Since every option has a default value, a Kairoi server can be started without any configuration file.

Here is an example of configuration file with all default options explicitly set:

```toml
[log]
level = "info" # One of "trace", "debug", "info", "warn", "error" or "off".

[controller]
listen = "127.0.0.1:5678" # You can use "0.0.0.0:5678" to accept connections from any client.

[database]
fsync_on_persist = true # Setting false can improve performances at the price of durability.
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

### Database

The `database` table contains all configuration options related to Kairoi's database, the component responsible for storing jobs and rules, and triggering job executions.

#### Fsync On Persist

`database.fsync_on_persist`: `Boolean` (default: `true`)

This option enables or disables the `fsync` operation on each data persist operation. Disabling it will prevent Kairoi to provide "Durability" (the D in ACID), but may improve performances a lot on some systems. It can be disabled in cases where all data written to Kairoi can be reconstructed from zero. When not sure, this option should be left to its default value.

#### Framerate

`database.framerate`: `Integer` (default: `512`)

This option configures the maximum framerate of the database component. The framerate is the number of cycles executed per second by the database. Only numbers between `1` and `65535` are valid. While it configures the maximum framerate (preventing to overcharge the CPU), the algorithm also tries to run the closest possible from this framerate. Increasing the framerate increases the rate at which the database handles write requests and triggers jobs. A value of `512` means 512 cycles per second, and so checking for write requests or jobs triggers should happens once every 2ms, bringing an average latency of 1ms. A value of `128` would bring the average latency to 4ms. This option should be set following your CPU availability: the more is the better, but also requiring more CPU.

## Internals

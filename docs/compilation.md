# Kairoi Compilation

## Quick Words

The first step before being able to use Kairoi is to compile it. The compilation process is made to be simple to use. The program can be compiled (providing default features) by cloning the repository and running `cargo build --release`. The executable can then be found at `target/release/kairoi`.

However, to provide useful features for Kairoi administrators, there may be times where advanced compilation can be used. The main way to customize Kairoi's compilation is by providing environment variables to the build command. For example, the default configuration file's path can be set using `CONFIGURATION_PATH="/etc/kairoi/configuration.toml" cargo build --release`. This will read the configuration from the file `/etc/kairoi/configuration.toml` instead of the default `./configuration.toml` that is relative to the launch directory.

## Usage

### Configuration Path

`CONFIGURATION_PATH` (default: `configuration.toml`)

This option sets the default configuration file's path used to read the configuration. It can receive either an absolute or a relative path. In case of a relative path, the path will be computed from the directory where the Kairoi server is launched.

If the configuration file doesn't exist, the program will start with the default configuration without any warning (read the [Kairoi Configuration Reference](configuration.md) for more informations on default values). However, if the file has an invalid format, Kairoi won't start at all.

```bash
# This will read the configuration from /etc/kairoi/configuration.toml
CONFIGURATION_PATH="/etc/kairoi/configuration.toml" cargo build --release && ./target/release/kairoi
# This will read the configuration from ./vars/configuration.toml
CONFIGURATION_PATH="vars/configuration.toml" cargo build --release && ./target/release/kairoi
```

## Internals

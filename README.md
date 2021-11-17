# Kairoi

**Kairoi** is a _Dynamic_, _Accurate_ and _Scalable_ **Time-based Job Scheduler** written in Rust.

## Quick Words

Kairoi is a **Time-based Job Scheduler**. It works as a server allowing its clients to schedule jobs to be executed in the future, using a simple text protocol (read more about the protocol in the [Kairoi Client Protocol documentation](docs/client-protocol.md)).

Once the job execution time is past, Kairoi automatically triggers a job execution on a matching configured runner (read more about runners in the [Kairoi Runners documentation](runners.md)). In its default configuration, Kairoi guarantees [ACID](https://en.wikipedia.org/wiki/ACID) properties on its transactions. Kairoi also uses a _at-least once_ delivery model: each job is guaranteed to be processed at-least once at some point after its execution date, but can also be processed more than one time. Thus, domain code handling jobs should be [idempotent](https://en.wikipedia.org/wiki/Idempotence).

Kairoi currently targets running on Linux operating systems.

## Documentation

* [Kairoi Official Documentation](docs/index.md)

## Development

Developping on Kairoi requires you to have [Rust installed on your machine](https://www.rust-lang.org/tools/install) in its latest version (`1.54.0` as of today).

### Installation

You can build the development version of Kairoi by cloning this repository, then using :

```
$> cargo build
```

at the root of this repository. It will automatically download and install all required dependencies.

## Installation using Docker

To use Kairoi with Docker, you can use the following Dockerfile to build and run a Kairoi server:

```Dockerfile

ARG RUST_VERSION=1

FROM rust:${RUST_VERSION}-alpine AS builder

RUN set -ex \
    && apk add --no-cache --virtual .build-deps git musl-dev openssl-dev \
    && git clone https://github.com/emerick42/kairoi.git /usr/src/kairoi \
	&& cd /usr/src/kairoi \
	&& cargo install --path .

WORKDIR /usr/src/kairoi

FROM alpine

COPY --from=builder /usr/local/cargo/bin/kairoi /usr/local/bin/kairoi

CMD ["kairoi"]

EXPOSE [5678]

```

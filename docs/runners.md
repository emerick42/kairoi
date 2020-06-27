# Kairoi Runners

## Quick Words

Each time a job execution is triggered, a runner is paired with this job, following existing rules in the database. Then the paired runner is executed, using the runner configuration from the rule. If the runner execution succeeds, the job is then marked as executed in the database. Otherwise, the job is marked as failed.

There are currently two existing runners: the `shell` runner and the `amqp` runner.

The `shell` runner works by executing a configured shell script in a separated thread with the job's identifier as first parameter, then changing the job status following the script return code. It's a simple runner to configure, but it does not allow for great scalability, since the script must be available on the filesystem used by the Kairoi server, and the job execution rate might be low.

`amqp` runners execute jobs by publishing messages to AMQP servers (typically RabbitMQ instances) with job identifiers as payload. Jobs are marked as executed as soon as messages are published to AMQP servers: waiting for AMQP workers to handle these messages is beyond the scope of Kairoi. This runner is a bit more difficult to configure since it also requires configuring an AMQP server, exchange and queue, but it is a great way to manage scalability: job execution is not limited to a single machine (it uses network to reach the AMQP server), and the job execution rate is much higher than when using the `shell` runner.

## Usage

### Shell

The `shell` runner is a simple runner, executing jobs using configured shell scripts. When paired with a job for execution, it executes the configured shell script with the job's identifier as first parameter, in a separated thread.

This runner currently supports a single configuration property: the path of the shell script or command to be used for job execution.

Since the script execution is triggered in a separated thread, it will not block the Kairoi server from running properly if executing a slow script. However, the strategy currently used for execution is to spawn a thread for each new script. Therefore, it is not recommended to use this runner when simultaneously running large numbers of jobs.

This runner is currently only compatible with Linux operating systems, since it uses the `sh` command to run these scripts.

#### Examples

Considering a script `script.sh` located in Kairoi's root directory:

```sh
#!/bin/sh

echo "Job $1 has been executed." > test.log
```

creating the rule:

```
SET RULE app.default.rule app.job.0 shell script.sh
SET app.job.0 "2020-06-26 16:48:00"
```

will have a result of `test.log` containing:

```
Job app.job.0 has been executed.
```

### AMQP

The `amqp` runner executes jobs by publishing messages to AMQP servers. Published messages contains the job identifier as payload. The runner publishes its messages to an AMQP server located at the configured data source name, on the configured exchange and with the configured routing key.

This runner currently supports three configuration properties, in this order:
* the data source name of the AMQP server, containing the URL, the couple of identifiers and the virtual host to use (for example `amqp://guest:guest@localhost:5672/`, for an AMQP server listening on `localhost:5672`, with identifier and password `guest`, and the virtual host `/`),
* the exchange name to publish to,
* and the routing key used to publish the message.

The `amqp` runner is currently the runner that can handle the highest number of job executions per second. For this, it uses kept-alive connections with AMQP servers. Since the rule model in Kairoi is highly dynamic (a rule can be added, modified or deleted at any time: read more in [the RULE SET instruction documentation](instructions.md#rule-set)), kept-alive connections cannot be open at server launch. They are instead open the first time the rule is paired with a job for execution. It leads to having the first job execution on a given AMQP connection taking more time than subsequent executions (the time to do the AMQP handshake and channel opening). Kairoi is configured to keep alive a fixed number of connections (currently 16). The oldest connection is dropped each time a new connection is opened and the memory is full. Thus, the usage of the AMQP runner should be avoided in a context with a large number of connections with different AMQP servers, and a large number of simultaneous job executions with different AMQP configurations.

#### Examples

```
SET RULE app.default.rule app. amqp amqp://login:password@localhost:5672/myvirtualhost my_exchange my_routing_key
SET RULE app.default.rule app. amqp amqp://my-rabbit@5672/ app_exchange app_kairoi
```

## Internals

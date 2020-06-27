# Kairoi Runners

## Quick Words

Each time a job execution is triggered, a runner is paired with this job, following existing rules in the database. Then the paired runner is executed, using the runner configuration from the rule. If the runner execution succeeds, the job is then marked as executed in the database. Otherwise, the job is marked as failed.

There is currently a single existing runner: the `shell` runner. It works by executing a configured shell script in a separated thread with the job's identifier as first parameter, then changing the job status following the script return code. It's a simple runner to configure, but it does not allow for great scalability, since the script must be available on the filesystem used by the Kairoi server, and the job execution rate might be low.

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

## Internals

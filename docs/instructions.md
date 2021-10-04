# Kairoi Instructions

## Quick Words

To communicate with Kairoi servers, clients must send instructions using the Kairoi Client Protocol (read more about this protocol in the [Kairoi Client Protocol documentation](client-protocol.md)). Instructions are defined by Kairoi servers and may evolve with versions.

Currently, there are two main instructions recognized by Kairoi servers:
* `SET identifier execution`: register a Job with the given identifier to be executed at the given execution time.
* `RULE SET identifier pattern runner [runner_arguments...]`: register a Rule with the given identifier, matching jobs with the given pattern, and executing the job with the given runner.

Here is a basic usage example, defining a default rule matching all jobs having identifiers starting by `app.` with the Shell runner configured to execute the file `script.sh`, then creating a job `app.domain.job.1` to be triggered at `2020-06-17 21:47:16 UTC`:

```
0 RULE SET app.rule.default app. shell script.sh
1 SET app.domain.job.1 "2020-06-17 21:47:16"
```

## Usage

### Job Set

```
SET identifier execution
```

with:
* `identifier`: any string, uniquely identifying a job,
* and `execution`: a date time in the UTC timezone, formatted like `Y-m-d H:i:s`.

This instruction registers a job with the given identifier to be triggered once the given execution time is past. If the execution time is in the past, the job will be triggered as soon as possible.

If a job with the given identifier is already set, it will update its execution time and status instead. This operation has different output depending on the current status of the job:
* for a job in status `Planned`, it will simply modify its execution time,
* for a job in statuses `Executed` or `Failed`, it will modify its execution time and set its status to `Planned`,
* and for a job in status `Triggered`, it will return an error.

#### Examples

```
0 SET app.domain.job.1 "2020-06-17 22:15:43"
1 SET "my emoji job \U+1F613" "2020-06-17 22:16:13"
```

### Rule Set

```
RULE SET identifier pattern runner [runner_arguments...]
```

with:
* `identifier`: any string, uniquely identifying this rule,
* `pattern`: any string, being the starts of job identifiers you want to match,
* `runner`: one of the existing runner kind (read more about runners in the [Kairoi Runners documentation](runners.md)),
* and optionnally multiple `runner_arguments`: a configuration element for the selected runner.

This instruction registers a rule with the given identifier. This rule will match triggered jobs having their identifier starting by the given pattern (the longer the matching pattern is, the higher is the rule priority). Once this rule is paired with a job (a job is triggered with this rule as the best match), it will execute the runner with the given configuration to handle this job.

#### Examples

```
0 RULE SET app.rule.default app. shell script.sh
1 RULE SET "my precise rule" "my emoji job \U+1F613" shell /bin/job_handler
```

## Internals

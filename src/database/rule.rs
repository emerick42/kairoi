use crate::execution::runner::Runner;

/// Rules associate String patterns to configured Runners. A pattern is a simple String (no special
/// character) matching all job identifiers starting with it. For example, the pattern "test." will
/// match the job "test.0", but won't match the job "test0".
#[derive(Clone, Debug)]
pub struct Rule {
    identifier: String,
    pattern: String,
    runner: Runner,
}

impl Rule {
    /// Create a new rule.
    pub fn new(identifier: String, pattern: String, runner: Runner) -> Rule {
        Rule {
            identifier: identifier,
            pattern: pattern,
            runner: runner,
        }
    }

    /// Check if the rule support the given job identifier. Return the weight of this rule. The
    /// higher the weight of a rule is, the highest should this rule be prioritized for execution.
    pub fn supports(&self, job: &String) -> Option<usize> {
        match job.starts_with(&self.pattern) {
            true => Some(self.pattern.len()),
            false => None,
        }
    }

    /// Get the identifier.
    pub fn get_identifier(&self) -> &String {
        &self.identifier
    }

    /// Get the runner.
    pub fn get_runner(&self) -> &Runner {
        &self.runner
    }
}

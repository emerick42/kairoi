#[derive(Clone, Debug)]
pub enum Runner {
    #[cfg(feature = "runner-shell")]
    Shell {
        command: String,
    },
}

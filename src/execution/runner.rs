#[derive(Clone, Debug)]
pub enum Runner {
    Shell {
        command: String,
    },
}

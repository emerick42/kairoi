#[derive(Clone, Debug)]
pub enum Runner {
    #[cfg(feature = "runner-shell")]
    Shell {
        command: String,
    },
    #[cfg(feature = "runner-amqp")]
    Amqp {
        dsn: String,
        exchange: String,
        routing_key: String,
    },
}

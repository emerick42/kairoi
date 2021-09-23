#[derive(Clone, Debug, PartialEq)]
pub enum Runner {
    Shell {
        command: String,
    },
    Amqp {
        dsn: String,
        exchange: String,
        routing_key: String,
    },
}

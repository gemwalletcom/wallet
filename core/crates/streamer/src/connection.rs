use std::error::Error;
use std::sync::Arc;

use lapin::uri::{AMQPScheme, AMQPUri};
use lapin::{Channel, Connection, ConnectionProperties};

#[derive(Clone)]
pub struct StreamConnection {
    url: String,
    name: String,
    connection: Arc<Connection>,
}

impl StreamConnection {
    pub async fn new(url: &str, name: impl Into<String>) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let name: String = name.into();
        let connection = connect(url, name.clone()).await?;
        Ok(Self {
            url: url.to_string(),
            name,
            connection: Arc::new(connection),
        })
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub async fn create_channel(&self) -> Result<Channel, Box<dyn Error + Send + Sync>> {
        Ok(self.connection.create_channel().await?)
    }
}

pub(crate) async fn connect(url: &str, name: String) -> Result<Connection, Box<dyn Error + Send + Sync>> {
    let options = ConnectionProperties::default().with_connection_name(name.into());
    Ok(Connection::connect_uri(plain_uri(url)?, options).await?)
}

fn plain_uri(url: &str) -> Result<AMQPUri, Box<dyn Error + Send + Sync>> {
    let uri: AMQPUri = url.parse()?;
    match uri.scheme {
        AMQPScheme::AMQP => Ok(uri),
        AMQPScheme::AMQPS => Err("amqps is not supported, lapin is built without TLS".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plain_uri() {
        let rejected = |url: &str| plain_uri(url).unwrap_err().to_string();

        assert_eq!(plain_uri("amqp://username:password@localhost:5672").unwrap().authority.port, 5672);
        assert_eq!(rejected("amqps://username:password@localhost:5671"), "amqps is not supported, lapin is built without TLS");
        assert_eq!(rejected("AMQPS://username:password@localhost:5671"), "amqps is not supported, lapin is built without TLS");
        assert_eq!(rejected("http://localhost"), "Invalid AMQP scheme: http");
    }
}

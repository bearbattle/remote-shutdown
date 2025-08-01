use std::{any::Any, collections::HashMap, error::Error, time::Duration};

use crate::config::{Config, Server};
use rumqttc::{
    Client, Connection, MqttOptions, Transport,
    tokio_rustls::{self, rustls::ClientConfig},
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
enum MqttServerConfig {
    Plain(PlainMqttServerConfig),
    TLS(TlsMqttServerConfig),
    WS(WsMqttServerConfig),
    WSS(WssMqttServerConfig),
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename = "config")]
pub(crate) struct PlainMqttServerConfig {
    host: String,
    port: u16,
    topic: String,
    uid: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename = "config")]
pub(crate) struct TlsMqttServerConfig {
    host: String,
    port: u16,
    topic: String,
    uid: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename = "config")]
pub(crate) struct WsMqttServerConfig {
    host: String,
    port: u16,
    topic: String,
    uid: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename = "config")]
pub(crate) struct WssMqttServerConfig {
    host: String,
    port: u16,
    topic: String,
    uid: String,
}

impl Default for MqttServerConfig {
    fn default() -> Self {
        Self::Plain(PlainMqttServerConfig::default())
    }
}

impl Config for MqttServerConfig {}

impl MqttServerConfig {
    fn uid(&self) -> &str {
        match self {
            Self::Plain(config) => &config.uid,
            Self::TLS(config) => &config.uid,
            Self::WS(config) => &config.uid,
            Self::WSS(config) => &config.uid,
        }
    }

    fn host(&self) -> &str {
        match self {
            Self::Plain(config) => &config.host,
            Self::TLS(config) => &config.host,
            Self::WS(config) => &config.host,
            Self::WSS(config) => &config.host,
        }
    }

    fn port(&self) -> u16 {
        match self {
            Self::Plain(config) => config.port,
            Self::TLS(config) => config.port,
            Self::WS(config) => config.port,
            Self::WSS(config) => config.port,
        }
    }

    fn topic(&self) -> &str {
        match self {
            Self::Plain(config) => &config.topic,
            Self::TLS(config) => &config.topic,
            Self::WS(config) => &config.topic,
            Self::WSS(config) => &config.topic,
        }
    }
}

struct MqttServer<'a> {
    config: &'a MqttServerConfig,
    handlers: HashMap<&'a str, &'a dyn Fn(&str) -> Result<Box<dyn Any>, Box<dyn Error>>>,
}

impl<'a> MqttServer<'a> {
    fn init(&mut self) -> (Client, Connection) {
        let uid = self.config.uid();
        let host = self.config.host();
        let port = self.config.port();
        let mut mqtt_options = MqttOptions::new(uid, host, port);
        match &self.config {
            MqttServerConfig::Plain(_) => {}
            MqttServerConfig::TLS(_) => {
                let mut root_cert_store = tokio_rustls::rustls::RootCertStore::empty();
                root_cert_store.add_parsable_certificates(
                    rustls_native_certs::load_native_certs()
                        .expect("could not load platform certs"),
                );
                let client_config = ClientConfig::builder()
                    .with_root_certificates(root_cert_store)
                    .with_no_client_auth();

                mqtt_options.set_transport(Transport::tls_with_config(client_config.into()));
            }
            MqttServerConfig::WS(_) => {
                mqtt_options.set_transport(Transport::Ws);
            }
            MqttServerConfig::WSS(_) => {
                let mut root_cert_store = tokio_rustls::rustls::RootCertStore::empty();
                root_cert_store.add_parsable_certificates(
                    rustls_native_certs::load_native_certs()
                        .expect("could not load platform certs"),
                );

                let client_config = ClientConfig::builder()
                    .with_root_certificates(root_cert_store)
                    .with_no_client_auth();
                mqtt_options.set_transport(Transport::wss_with_config(client_config.into()));
            }
        }
        mqtt_options.set_keep_alive(Duration::new(5, 0));

        Client::new(mqtt_options, 1)
    }
}

impl<'a> Server<'a, MqttServerConfig> for MqttServer<'a> {
    fn new(config: &'a MqttServerConfig) -> Self {
        Self {
            config,
            handlers: HashMap::new(),
        }
    }

    fn register_handler(
        &mut self,
        keyword: &'a str,
        handler: &'a dyn Fn(&str) -> Result<Box<dyn Any>, Box<dyn Error>>,
    ) {
        self.handlers.insert(keyword, handler);
    }

    fn run_loop(&mut self) -> Result<(), Box<dyn Error>> {
        let topic = self.config.topic().to_string();

        let (client, mut connection) = self.init();

        client.subscribe(&topic, rumqttc::QoS::AtLeastOnce)?;

        loop {
            if let Some(notification) = connection.iter().next() {
                dbg!("Notification = {:?}", notification);
                // TODO: 实现消息处理逻辑，调用已注册的处理程序
            }
        }
    }
}

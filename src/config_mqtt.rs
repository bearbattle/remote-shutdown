// remote-shutdown: Use TCP/MQTT to shutdown device
// Copyright (C) 2025 Bear Battle
use std::{collections::HashMap, error::Error, time::Duration};

use crate::config::{Callback, Server};
use rumqttc::{
    Client, Connection,
    Event::Incoming,
    MqttOptions, Transport,
    tokio_rustls::{self, rustls::ClientConfig},
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum MqttServerConfig {
    Plain(PlainMqttServerConfig),
    TLS(TlsMqttServerConfig),
    WS(WsMqttServerConfig),
    WSS(WssMqttServerConfig),
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename = "config")]
pub struct PlainMqttServerConfig {
    host: String,
    port: u16,
    topic: String,
    uid: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename = "config")]
pub struct TlsMqttServerConfig {
    host: String,
    port: u16,
    topic: String,
    uid: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename = "config")]
pub struct WsMqttServerConfig {
    host: String,
    port: u16,
    topic: String,
    uid: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename = "config")]
pub struct WssMqttServerConfig {
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

pub struct MqttServer {
    config: MqttServerConfig,
    handlers: HashMap<String, Callback>,
}

impl MqttServer {
    pub fn new(config: MqttServerConfig) -> Box<dyn Server> {
        Box::new(Self {
            config: config,
            handlers: HashMap::new(),
        })
    }
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

impl Server for MqttServer {
    fn register_handler(&mut self, keyword: &str, handler: Callback) {
        self.handlers.insert(keyword.to_string(), handler);
    }

    fn run_loop(&mut self) -> Result<(), Box<dyn Error>> {
        let topic = self.config.topic().to_string();

        let (client, mut connection) = self.init();

        client.subscribe(&topic, rumqttc::QoS::AtLeastOnce)?;

        loop {
            if let Some(notification) = connection.iter().next() {
                dbg!("Notification = {:?}", &notification);
                if let Ok(msg) = notification
                    && let Incoming(packet) = msg
                {
                    match packet {
                        rumqttc::Packet::Publish(publish) => {
                            let payload = String::from_utf8(publish.payload.to_vec())?;
                            dbg!(&payload);
                            let msg_map: HashMap<&str, &str> = payload
                                .trim()
                                .split('&')
                                .map(|x| {
                                    let entry: Vec<&str> = x.split('=').collect();
                                    (entry[0], *entry.get(1).unwrap_or(&""))
                                })
                                .collect();
                            for (k, v) in msg_map {
                                if let Some(handler) = self.handlers.get(k) {
                                    if let Err(e) = handler(v) {
                                        return Err(e);
                                    };
                                }
                            }
                        }
                        other => {
                            dbg!(other);
                        }
                    }
                }
            }
        }
    }
}

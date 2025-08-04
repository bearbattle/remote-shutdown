use std::{any::Any, error::Error};

use serde::{Deserialize, Serialize};

pub type Callback = fn(&str) -> Result<Box<dyn Any>, Box<dyn Error>>;

pub trait Server {
    fn register_handler(&mut self, keyword: &str, handler: Callback);
    fn run_loop(&mut self) -> Result<(), Box<dyn Error>>;
}

use crate::config_mqtt::{
    PlainMqttServerConfig, TlsMqttServerConfig, WsMqttServerConfig, WssMqttServerConfig,
};
use crate::config_tcp::TcpServerConfig;

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "proto", content = "config")]
pub enum LocalConfig {
    #[serde(rename = "tcp")]
    Tcp(TcpServerConfig),
    #[serde(rename = "mqtt")]
    PlainMqtt(PlainMqttServerConfig),
    #[serde(rename = "mqtt-tls")]
    TlsMqtt(TlsMqttServerConfig),
    #[serde(rename = "mqtt-ws")]
    WsMqtt(WsMqttServerConfig),
    #[serde(rename = "mqtt-wss")]
    WssMqtt(WssMqttServerConfig),
}

#[cfg(test)]
mod tests {
    use super::*;
    // use unescaper::unescape as un;

    #[test]
    fn test_tcp_server_config() {
        // Test Serialize
        let config = LocalConfig::Tcp(TcpServerConfig::default());
        println!("{}", toml::to_string_pretty(&config).unwrap());
        // Test Deserialize
        let import_config = toml::from_str::<LocalConfig>(
            r#" 
proto = "tcp"
[config]
host = "localhost"
port = 1234
topic = "light"
uid = "123456789"
"#,
        )
        .unwrap();
        println!("{:#?}", import_config);
    }

    #[test]
    fn test_mqtt_server_config() {
        // Test Serialize

        let config = LocalConfig::PlainMqtt(PlainMqttServerConfig::default());

        println!("{}", toml::to_string_pretty(&config).unwrap());
        // Test Deserialize

        let import_config = toml::from_str::<LocalConfig>(
            r#"
proto = "mqtt"
[config]
host = "localhost"
port = 1234
topic = "light"
uid = "123456789"
"#,
        );
        println!("{:#?}", import_config);
    }
}

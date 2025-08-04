// remote-shutdown: Use TCP/MQTT to shutdown device
// Copyright (C) 2025 Bear Battle
mod config;
mod config_mqtt;
mod config_tcp;
mod handler;

use std::error::Error;

use config::LocalConfig;
use config_file::FromConfigFile;

use crate::{
    config::Server,
    config_mqtt::{MqttServer, MqttServerConfig},
    config_tcp::TcpServer,
};

fn main() -> Result<(), Box<dyn Error>> {
    let config = LocalConfig::from_config_file("config.toml")?;
    let mut server: Box<dyn Server> = match config {
        LocalConfig::Tcp(config) => TcpServer::new(config),
        LocalConfig::PlainMqtt(config) => MqttServer::new(MqttServerConfig::Plain(config)),
        LocalConfig::TlsMqtt(config) => MqttServer::new(MqttServerConfig::TLS(config)),
        LocalConfig::WsMqtt(config) => MqttServer::new(MqttServerConfig::WS(config)),
        LocalConfig::WssMqtt(config) => MqttServer::new(MqttServerConfig::WSS(config)),
    };
    server.register_handler("off", handler::shutdown_system);
    server.run_loop()?;
    Ok(())
}

// remote-shutdown: Use TCP/MQTT to shutdown device
// Copyright (C) 2025 Bear Battle
use std::{
    collections::HashMap,
    error::Error,
    io::{Read, Write},
    net::TcpStream,
    time::Duration,
};

use crate::config::{Callback, Server};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct TcpServerConfig {
    host: String,
    port: u16,
    topic: String,
    uid: String,
}

pub struct TcpServer {
    config: TcpServerConfig,
    handlers: HashMap<String, Callback>,
}

impl TcpServer {
    pub fn new(config: TcpServerConfig) -> Box<dyn Server> {
        Box::new(Self {
            config,
            handlers: HashMap::new(),
        })
    }

    fn init_stream(&self) -> Result<TcpStream, Box<dyn Error>> {
        let mut stream =
            TcpStream::connect(format!("{}:{}", &self.config.host, &self.config.port))?;
        dbg!("TCP connected, trying to subscribe to topic...");
        stream.set_write_timeout(Some(Duration::new(1, 0)))?;
        stream.set_read_timeout(Some(Duration::new(60, 0)))?;
        stream.write_all(
            format!(
                "cmd=1&uid={}&topic={}\r\n",
                &self.config.uid, &self.config.topic
            )
            .as_bytes(),
        )?;
        let mut msg = [0 as u8; 1024];
        let msg_len = stream.read(&mut msg)?;
        assert_eq!(
            &msg[0..msg_len],
            "cmd=1&res=1\r\n".as_bytes(),
            "Failed to subscribe to topic!"
        );
        Ok(stream)
    }

    fn event_loop(&self, stream: &mut TcpStream) -> Result<(), Box<dyn Error>> {
        loop {
            let mut msg = [0 as u8; 1024];
            match stream.read(&mut msg) {
                Ok(msg_len) => {
                    let full_str = String::from_utf8(msg[0..msg_len].to_vec())?;
                    let full_str = full_str.trim();
                    let msg_str_start_index = full_str.find("msg=");
                    if None == msg_str_start_index {
                        dbg!("Received non-message data, ignoring...");
                        continue;
                    }
                    let msg_str_start_index = msg_str_start_index.unwrap() + 4; // Skip "msg="
                    let msg_str = &full_str[msg_str_start_index..];
                    dbg!(&msg_str);
                    let msg_map: HashMap<&str, &str> = msg_str
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
                Err(err) => {
                    #[cfg(not(target_os = "windows"))]
                    if err.kind() == std::io::ErrorKind::WouldBlock {
                        // Read timeout, sending Heartbeat...
                        dbg!("Sending heartbeat...");
                        write!(stream, "ping\r\n")?;
                    }
                    #[cfg(target_os = "windows")]
                    if err.kind() == std::io::ErrorKind::TimedOut {
                        // Read timeout, sending Heartbeat...
                        dbg!("Sending heartbeat...");
                        write!(stream, "ping\r\n")?;
                    }
                    return Err(Box::new(err));
                }
            }
        }
    }
}

impl Server for TcpServer {
    fn register_handler(&mut self, keyword: &str, handler: Callback) {
        self.handlers.insert(keyword.to_string(), handler);
    }

    fn run_loop(&mut self) -> Result<(), Box<dyn Error>> {
        loop {
            let mut stream = self.init_stream()?;
            self.event_loop(&mut stream)?;
        }
    }
}

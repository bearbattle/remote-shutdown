use std::{
    error::Error,
    io::{Read, Write},
    net::TcpStream,
    time::Duration,
};

use serde::{Deserialize, Serialize};

use config_file::FromConfigFile;

use system_shutdown::shutdown;

#[derive(Serialize, Deserialize, Debug)]
struct TcpServerConfig {
    host: String,
    port: u16,
    topic: String,
    uid: String,
}

impl ::std::default::Default for TcpServerConfig {
    fn default() -> Self {
        Self {
            host: String::from("localhost"),
            port: 0,
            topic: String::from(""),
            uid: String::from(""),
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let conf: TcpServerConfig = TcpServerConfig::from_config_file("config.toml")?;
    dbg!(&conf);
    let mut stream = TcpStream::connect(format!("{}:{}", &conf.host, &conf.port))?;
    dbg!("TCP connected, trying to subscribe to topic...");
    stream.set_write_timeout(Some(Duration::new(1, 0)))?;
    stream.set_read_timeout(Some(Duration::new(60, 0)))?;
    stream.write_all(format!("cmd=1&uid={}&topic={}\r\n", &conf.uid, &conf.topic).as_bytes())?;
    let mut msg = [0 as u8; 1024];
    let msg_len = stream.read(&mut msg)?;
    assert_eq!(
        &msg[0..msg_len],
        "cmd=1&res=1\r\n".as_bytes(),
        "Failed to subscribe to topic!"
    );
    dbg!("Subscribed for topic!");
    loop {
        let mut msg = [0 as u8; 1024];
        match stream.read(&mut msg) {
            Ok(msg_len) => {
                let msg_str = String::from_utf8(msg[0..msg_len].to_vec())?;
                dbg!(&msg_str);
                if let Some(msg_index) = msg_str.find("msg") {
                    if let Some(act) = msg_str.get((msg_index + 4)..) {
                        if act.split(|c| c == '\r' || c == '&').next() == Some("off") {
                            dbg!("Received off command");
                            stream.shutdown(std::net::Shutdown::Both)?;
                            #[cfg(not(debug_assertions))]
                            shutdown()?;
                            break;
                        }
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
    Ok(())
}

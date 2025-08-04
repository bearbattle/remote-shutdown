// remote-shutdown: Use TCP/MQTT to shutdown device
// Copyright (C) 2025 Bear Battle
#[allow(unused_imports)]
use std::{any::Any, error::Error, process::exit};

#[cfg(not(debug_assertions))]
#[allow(unused_imports)]
use system_shutdown::shutdown;

pub(crate) fn shutdown_system(msg: &str) -> Result<Box<dyn Any>, Box<dyn Error>> {
    dbg!("Shutting down...", msg);
    #[cfg(not(debug_assertions))]
    shutdown()?;
    #[cfg(debug_assertions)]
    {
        println!("Shutting down...");
        exit(0);
    };
    #[cfg(not(debug_assertions))]
    Ok(Box::new(()))
}

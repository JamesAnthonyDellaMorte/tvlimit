use serde_json::{json, Value};
use std::net::TcpStream;
use std::{thread, time};
#[derive(PartialEq)]
pub enum PlugState {
    On,
    Off,
    Idle,
    Unknown,
}

pub struct SmartPlug {
    host_ip: String,
    pub state: PlugState,
}
impl SmartPlug {
    pub fn new(ip: String) -> Self {
        Self {
            host_ip: ip,
            state: PlugState::Unknown,
        }
    }
    pub fn get_amps(&self) -> f32 {
        let raw = r#"{"emeter":{"get_realtime":{}}}"#;
        let stream = loop {
            match TcpStream::connect(&self.host_ip) {
                Ok(s) => break s,
                Err(e) => {
                    println!("Could not connect due to {e} trying again in 10");
                    thread::sleep(time::Duration::from_secs(10));
                }
            }
        };
        loop {
            match tplink_shome_protocol::send_message(&stream, raw) {
                Ok(_) => break,
                Err(_) => {
                    println!("Could not send message to get amps! trying again");
                    thread::sleep(time::Duration::from_secs(10));
                }
            }
        };
        let message =
            tplink_shome_protocol::receive_message(&stream).unwrap_or_else(|_| String::from("0.0"));
        let emeter: Value = serde_json::from_str(&message).unwrap_or(json!(null));
        let current_ma = emeter["emeter"]["get_realtime"]["current_ma"]
            .as_u64()
            .unwrap_or(u64::MAX);
        if current_ma == u64::MAX {
            println!("Could not connect to device trying again in 10 secs");
            thread::sleep(time::Duration::from_secs(10));
        }
        (current_ma as f32) / 1000.0
    }
    pub fn update_state(&mut self) {
        if self.get_amps() > 0.5 {
            self.state = PlugState::On;
        } else if self.get_amps() == 0.0 {
            self.state = PlugState::Off;
        } else {
            self.state = PlugState::Idle;
        }
    }
    pub fn on(&self) {
        println!("Turning on TV");
        let raw = r#"{"system":{"set_relay_state":{"state":1}}}"#;
        let stream = match TcpStream::connect(&self.host_ip) {
            Ok(s) => s,
            Err(e) => {
                println!("Could not connect due to {} trying again in 10", e);
                thread::sleep(time::Duration::from_secs(10));
                TcpStream::connect(&self.host_ip).unwrap()
            }
        };
        tplink_shome_protocol::send_message(&stream, raw).expect("msg could not send");
        let rst = tplink_shome_protocol::send_message(&stream, raw);
        match rst {
            Ok(_) => (),
            Err(_) => {
                println!("Could not send message to turn on device! trying again");
                thread::sleep(time::Duration::from_secs(10));
                self.on();
            }
        }
    }
    pub fn off(&self) {
        println!("Turning off TV");
        let raw = r#"{"system":{"set_relay_state":{"state":0}}}"#;
        let stream = match TcpStream::connect(&self.host_ip) {
            Ok(s) => s,
            Err(e) => {
                println!("Could not connect due to {e} trying again in 10");
                thread::sleep(time::Duration::from_secs(10));
                TcpStream::connect(&self.host_ip).unwrap()
            }
        };
        let rst = tplink_shome_protocol::send_message(&stream, raw);
        match rst {
            Ok(_) => (),
            Err(_) => {
                println!("Could not send message to turn off device! trying again");
                thread::sleep(time::Duration::from_secs(10));
                self.off();
            }
        }
    }
}

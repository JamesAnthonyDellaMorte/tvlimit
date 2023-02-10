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
    pub state: PlugState,
    stream: TcpStream,
}
impl SmartPlug {
    pub fn new(host_ip: String) -> Self {
        Self {
            state: PlugState::Unknown,
            stream: TcpStream::connect(host_ip).expect("Could not set up TCPStream"),
        }
    }
    pub fn get_amps(&self) -> f32 {
        let raw = r#"{"emeter":{"get_realtime":{}}}"#;
        tplink_shome_protocol::send_message(&self.stream, raw).expect("msg could not send");
        let message = tplink_shome_protocol::receive_message(&self.stream)
            .unwrap_or_else(|_| String::from("0.0"));
        let emeter: Value = serde_json::from_str(&message).unwrap_or(json!(null));
        let current_ma = emeter["emeter"]["get_realtime"]["current_ma"]
            .as_u64()
            .unwrap_or(u64::MAX);
        if current_ma == u64::MAX {
            println!("Could not connect to device trying again in 10 secs");
            thread::sleep(time::Duration::from_secs(10));
            self.get_amps();
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
        tplink_shome_protocol::send_message(&self.stream, raw).expect("msg could not send");
    }
    pub fn off(&self) {
        println!("Turning off TV");
        let raw = r#"{"system":{"set_relay_state":{"state":0}}}"#;
        tplink_shome_protocol::send_message(&self.stream, raw).expect("msg could not send");
    }
}

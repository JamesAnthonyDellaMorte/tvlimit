use serde::Deserialize;
use serde_json::{json, Value};
use std::net::TcpStream;
use std::{thread, time};
#[derive(PartialEq, Debug)]
pub enum TVState {
    On,
    Off,
    Idle,
    Play,
    Unknown,
}
#[derive(Debug, Deserialize)]
struct Plugin {
    // Example fields for the Plugin struct
    name: String,
}

// Implementing Default for Plugin just as an example
impl Default for Plugin {
    fn default() -> Self {
        Plugin {
            name: "Unknown".to_string(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct Player {
    #[serde(rename = "state")]
    state: String,
    plugin: Option<Plugin>, // Note the use of Option here
}

// Implement the Default trait for Player
impl Default for Player {
    fn default() -> Self {
        Player {
            state: "Unknown".to_string(),    // Provide a default state
            plugin: Some(Plugin::default()), // Use Some if you want a default plugin, or None to have no plugin by default
        }
    }
}
pub struct SmartPlug {
    host_ip: String,
    pub state: TVState,
    pub whats_playing: String,
}
impl SmartPlug {
    pub fn new(ip: String) -> Self {
        Self {
            host_ip: ip,
            state: TVState::Unknown,
            whats_playing: "Nothing".to_string(),
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
        }
        let message =
            tplink_shome_protocol::receive_message(&stream).unwrap_or(String::from("0.0"));
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
    pub fn player_state(&mut self) -> String {
        let response = ureq::get("http://10.0.0.49:8060/query/media-player").call();

        let body = match response {
            Ok(s) => s.into_reader(),
            Err(_) => {
                return "Off".to_string();
            }
        };
        let player: Player = serde_xml_rs::from_reader(body).unwrap_or(Player::default());
        self.whats_playing = match player.plugin {
            Some(s) => s.name,
            None => "Nothing".to_string(),
        };
        player.state.to_ascii_lowercase()
    }
    pub fn update_state(&mut self) {
        if self.player_state() == "play" {
            self.state = TVState::Play;
        } else if self.get_amps() == 0.0 {
            self.state = TVState::Off;
        } else if self.get_amps() > 0.5 {
            self.state = TVState::On;
        } else {
            self.state = TVState::Idle;
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
    pub fn press_home_button(&self) {
        let url = "http://10.0.0.49:8060/keypress/home";
        match ureq::post(url).call() {
            Ok(_) => println!("Pressed Home Button"),
            Err(e) => println!("Error pressing Home Button: {}", e),
        };
    }
}

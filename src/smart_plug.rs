use serde::Deserialize;
use std::net::TcpStream;
use std::{fmt, thread, time};
#[derive(PartialEq, Debug)]
pub enum TVState {
    Pause,
    Idle,
    Play,
    Unknown,
}

impl fmt::Display for TVState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TVState::Pause => write!(f, "Pause"),
            TVState::Idle => write!(f, "Idle"),
            TVState::Play => write!(f, "Play"),
            TVState::Unknown => write!(f, "Unknown"),
        }
    }
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
    plug_ip: String,
    roku_ip: String,
    pub state: TVState,
    pub whats_playing: String,
}
impl SmartPlug {
    pub fn new(plug_ip: String, roku_ip: String) -> Self {
        Self {
            plug_ip,
            roku_ip,
            state: TVState::Unknown,
            whats_playing: "Nothing".to_string(),
        }
    }
    pub fn player_state(&mut self) -> String {
        let response = ureq::get(&format!("http://{}/query/media-player", self.roku_ip)).call();

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
        } else if self.player_state() == "pause" {
            self.state = TVState::Pause;
        } else {
            self.state = TVState::Idle;
        }
    }
    pub fn on(&self) {
        println!("Turning on TV");
        let raw = r#"{"system":{"set_relay_state":{"state":1}}}"#;
        let stream = match TcpStream::connect(&self.plug_ip) {
            Ok(s) => s,
            Err(e) => {
                println!("Could not connect due to {} trying again in 10", e);
                thread::sleep(time::Duration::from_secs(10));
                TcpStream::connect(&self.plug_ip).unwrap()
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
        let stream = match TcpStream::connect(&self.plug_ip) {
            Ok(s) => s,
            Err(e) => {
                println!("Could not connect due to {e} trying again in 10");
                thread::sleep(time::Duration::from_secs(10));
                TcpStream::connect(&self.plug_ip).unwrap()
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
        match ureq::post(&format!("http://{}/keypress/home", self.roku_ip)).call() {
            Ok(_) => println!("Pressed Home Button"),
            Err(e) => println!("Error pressing Home Button: {}", e),
        };
    }
}

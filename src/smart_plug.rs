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
    name: String,
}

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
    plugin: Option<Plugin>,
}

impl Default for Player {
    fn default() -> Self {
        Player {
            state: "Unknown".to_string(),
            plugin: Some(Plugin::default()),
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
            Err(e) => {
                eprintln!("Error querying media player: {}", e);
                return "Off".to_string();
            }
        };

        match serde_xml_rs::from_reader(body) {
            Ok(player) => {
                let player: Player = player;
                self.whats_playing = player.plugin.map_or("Nothing".to_string(), |p| p.name);
                player.state.to_ascii_lowercase()
            }
            Err(e) => {
                eprintln!("Error parsing XML: {}", e);
                "Unknown".to_string()
            }
        }
    }

    pub fn update_state(&mut self) {
        self.state = match self.player_state().as_str() {
            "play" => TVState::Play,
            "pause" => TVState::Pause,
            _ => TVState::Idle,
        };
    }

    pub fn on(&self) {
        println!("Turning on TV");
        let raw = r#"{"system":{"set_relay_state":{"state":1}}}"#;
        self.send_command(raw, "turn on");
    }

    pub fn off(&self) {
        println!("Turning off TV");
        let raw = r#"{"system":{"set_relay_state":{"state":0}}}"#;
        self.send_command(raw, "turn off");
    }

    fn send_command(&self, raw: &str, action: &str) {
        let mut attempts = 0;
        while attempts < 3 {
            match TcpStream::connect(&self.plug_ip) {
                Ok(stream) => {
                    match tplink_shome_protocol::send_message(&stream, raw) {
                        Ok(_) => {
                            println!("Successfully sent command to {} device", action);
                            return;
                        }
                        Err(e) => {
                            eprintln!("Error sending message to {} device: {}", action, e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Could not connect: {}", e);
                }
            }
            attempts += 1;
            println!("Retrying in 10 seconds...");
            thread::sleep(time::Duration::from_secs(10));
        }
        eprintln!("Failed to {} device after 3 attempts", action);
    }

    pub fn press_home_button(&self) {
        match ureq::post(&format!("http://{}/keypress/home", self.roku_ip)).call() {
            Ok(_) => println!("Pressed Home Button"),
            Err(e) => eprintln!("Error pressing Home Button: {}", e),
        };
    }
}
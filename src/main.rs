use crate::smart_plug::SmartPlug;
use chrono::{DateTime, Local, Timelike};
use serde::Deserialize;
use std::time::{self, Duration};
use std::{fs, thread};
use humantime::{format_duration, parse_duration};
mod smart_plug;
#[derive(Deserialize, Clone)]
struct Settings {
    roku_ip: String,
    plug_ip: String,
    ttw: String,
}

#[derive(Deserialize, Clone)]
struct Config {
    user: Settings,
}
fn main() {
    let toml_str = fs::read_to_string("config.toml").unwrap_or("None".to_owned());
    let config: Result<Config, toml::de::Error> = toml::from_str(&toml_str);
    let mut plug: SmartPlug = match config.clone() {
        Ok(c) => {
            println!("Setting ip's from config!");
            smart_plug::SmartPlug::new(
                format!("{}:9999", c.user.plug_ip),
                format!("{}:8060", c.user.roku_ip),
            )
        }
        Err(_) => smart_plug::SmartPlug::new(
            String::from("10.0.0.44:9999"),
            String::from("10.0.0.124:8060"),
        ),
    };
    let ttw = match config {
        Ok(c) => Some(c.user.ttw),
        Err(_) => None,
    };
    loop {
        run_loop(&mut plug, ttw.clone());
        plug.off();
        wait_till_1230();
        plug.on();
    }
}

fn wait_till_1230() {
    let now = Local::now();
    let target_time = now
        .with_hour(12)
        .and_then(|time| time.with_minute(30))
        .and_then(|time| time.with_second(0));

    match target_time {
        Some(target_time) => {
            let sleep_delta = if target_time > now {
                target_time - now
            } else {
                target_time + time::Duration::from_secs(86400) - now
            };
            let sleep_duration = match sleep_delta.to_std()
            {
                Ok(d) => d,
                Err(_) => {
                    eprintln!("Failed to convert sleep delta to std duration.");
                    time::Duration::from_secs(1)
                }
            };
            

            println!("We are waiting until 12:30 pm to turn on TV! There are {} left.", format_duration(sleep_duration));

            thread::sleep(sleep_duration);
            println!("It's 12:30 pm!");
        }
        None => eprintln!("Failed to set target time."),
    }
}

fn run_loop(p: &mut smart_plug::SmartPlug, ttw: Option<String>) {
    let mut can_watch_tv = true;
    let local: DateTime<Local> = Local::now();
    let today = local.format("%A").to_string();
    let mut timer = parse_duration(&fs::read_to_string("tvtimer.txt")
        .unwrap_or("0\n".to_owned())
        .trim()
        .parse::<String>()
        .unwrap_or("0".to_owned())).unwrap_or(Duration::new(0, 0));
    let wait_for = match ttw {
        Some(v) => parse_duration(&v).unwrap_or(Duration::new(0, 0)),
        None => {
            if today == "Saturday" || today == "Sunday" {
                Duration::new(5400, 0)
            } else {
                Duration::new(3600, 0)
            }
        }
    };

        println!("Waiting for {}", format_duration(wait_for));
    if timer.is_zero(){
        println!("Starting timer at {} per text file" , format_duration(timer));
    }
    p.update_state();
    while can_watch_tv {
        match p.state {
            smart_plug::TVState::Play => {
                if (timer.as_secs() % 30) == 0 {
                    let delta = wait_for - timer;
                    println!(
                        "TV is Playing! Timer is at {}; {} of TV time left; - Looks like something on {} is playing",
                        format_duration(timer),
                        format_duration(delta),
                        p.whats_playing
                    );
                    fs::write("tvtimer.txt", timer.as_secs().to_string()).unwrap_or(());
                }
                thread::sleep(time::Duration::from_secs(1));
                timer += Duration::new(1, 0);
                p.update_state();
                if smart_plug::TVState::Play != p.state {
                    println!("State Change from Play! State is now {}", p.state);
                }
            }
            smart_plug::TVState::Pause => {
                println!("Looks like something on {} is paused, timer is at {}", p.whats_playing, format_duration(timer));
                while smart_plug::TVState::Pause == p.state {
                    thread::sleep(time::Duration::from_secs(1));
                    p.update_state();
                }
                println!("State Change from On! State is now {}", p.state);
            }
            smart_plug::TVState::Idle => {
                println!("No media is playing via a channel, timer is at {}",  format_duration(timer));
                while smart_plug::TVState::Idle == p.state {
                    let local: DateTime<Local> = Local::now();
                    let hrs = local.hour();
                    if hrs >= 21 {
                        can_watch_tv = false;
                        timer = Duration::new(0, 0);
                        fs::write("tvtimer.txt", timer.as_secs().to_string()).unwrap_or(());
                        break;
                    }
                    thread::sleep(time::Duration::from_secs(1));
                    p.update_state();
                }
                println!("State Change from Off! State is now {}", p.state);
            }
            smart_plug::TVState::Unknown => {
                p.update_state();
            }
        }
        if timer > wait_for {
            can_watch_tv = false;
            println!("The value has been true for {}" ,  format_duration(timer));
            timer = Duration::new(0, 0);
            fs::write("tvtimer.txt", timer.as_secs().to_string()).unwrap_or(());
            p.press_home_button();
            thread::sleep(time::Duration::from_secs(1));
        }
    }
}

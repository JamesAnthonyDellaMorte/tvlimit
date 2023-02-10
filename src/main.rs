use chrono::{DateTime, Local, Timelike};
use std::{process::Command, time};

use std::{fs, thread};
fn main() {
    let mut plug = SmartPlug::new(String::from("10.0.0.44"));
    println!("Amps at startup: {} A", plug.get_amps());

    loop {
        run_loop(&mut plug);
        wait_till_6();
        plug.on();
    }
}
fn wait_till_6() {
    let now = Local::now();
    let target_time = now
        .with_hour(6)
        .unwrap()
        .with_minute(0)
        .unwrap()
        .with_second(0)
        .unwrap();
    let sleep_duration = target_time.signed_duration_since(now);
    thread::sleep(time::Duration::from_secs(
        sleep_duration.num_seconds() as u64
    ));
}
fn run_loop(p: &mut SmartPlug) {
    let mut flag = true;
    let local: DateTime<Local> = Local::now();
    let mut today = local.format("%A").to_string();
    let mut timer = fs::read_to_string("tvtimer.txt")
        .unwrap_or_else(|_| "0\n".to_owned())
        .trim()
        .parse::<i32>()
        .unwrap_or(0);
    let mut wait_for = if today == "Saturday" || today == "Sunday" {
        10800
    } else {
        3600
    };

    println!("Waiting for {} hrs", wait_for / 3600);
    if timer != 0 
    {
        println!("Starting timer at {} per text file", timer);
    }
    while flag {
        match p.state {
            PlugState::On => {
                if (timer % 60) == 0 {
                    println!("TV is on! Timer is at {} mins", timer / 60);
                    fs::write("tvtimer.txt", timer.to_string()).unwrap_or(());
                }
                thread::sleep(time::Duration::from_secs(1));
                timer += 1;
                p.update_state();
            }
            PlugState::Off => {
                println!("TV is off,checking if 6 am");
                while PlugState::Off == p.state {
                    let local: DateTime<Local> = Local::now();
                    let hrs = local.hour();
                    if hrs == 6 {
                        p.on();
                        timer = 0;
                        fs::write("tvtimer.txt", timer.to_string()).unwrap_or(());
                    }
                    thread::sleep(time::Duration::from_secs(1));
                    p.update_state();
                }
                println!("State Change from Off!");
            }
            PlugState::Idle => {
                println!("TV is idle, timer is at {} secs", timer);
                while PlugState::Idle == p.state {
                    thread::sleep(time::Duration::from_secs(1));
                    let local: DateTime<Local> = Local::now();
                    let hrs = local.hour();
                    let secs = local.second();
                    if hrs == 6 && secs < 5 {
                        println!("A new day without all TV time being used!");
                        thread::sleep(time::Duration::from_secs(10));
                        timer = 0;
                        fs::write("tvtimer.txt", timer.to_string()).unwrap_or(());
                        today = local.format("%A").to_string();
                        wait_for = if today == "Saturday" || today == "Sunday" {
                            10800
                        } else {
                            3600
                        };
                        println!("Waiting for {} hrs", wait_for / 3600);
                    }

                    p.update_state();
                }
                println!("State Change from Idle!");
            }
            PlugState::Unknown => {
                p.update_state();
            }
        }
        if timer > wait_for {
            flag = false;
            p.off();
        }
    }
    println!("The value has been true for {} secs", timer);
}
#[derive(PartialEq)]
enum PlugState {
    On,
    Off,
    Idle,
    Unknown,
}

pub struct SmartPlug {
    host: String,
    state: PlugState,
}
impl SmartPlug {
    pub fn new(host_ip: String) -> Self {
        Self {
            host: host_ip,
            state: PlugState::Unknown,
        }
    }
    fn get_amps(&self) -> f32 {
        let out = Command::new("kasa")
            .arg("--host")
            .arg(&self.host)
            .arg("emeter")
            .output()
            .expect("kasa command failed to start");
        let rst = String::from_utf8(out.stdout).unwrap();
        let split = rst.split('\n');
        let vec: Vec<&str> = split.collect();
        if vec.len() > 2 {
            let cur_str = vec[2];
            let val: Vec<&str> = cur_str.split_whitespace().collect();
            let cur = val[1].parse::<f32>();
            cur.unwrap_or(0.0)
        } else {
            println!("Could not get amps, trying again!");
            thread::sleep(time::Duration::from_secs(10));
            self.get_amps()
        }
    }
    fn update_state(&mut self) {
        if self.get_amps() > 0.5 {
            self.state = PlugState::On;
        } else if self.get_amps() == 0.0 {
            self.state = PlugState::Off;
        } else {
            self.state = PlugState::Idle;
        }
    }
    fn on(&self) {
        println!("Turning on TV");
        Command::new("kasa")
            .arg("--host")
            .arg(&self.host)
            .arg("on")
            .output()
            .expect("kasa command failed to start");
    }
    fn off(&self) {
        println!("Turning off TV");
        Command::new("kasa")
            .arg("--host")
            .arg(&self.host)
            .arg("off")
            .output()
            .expect("kasa command failed to start");
    }
}

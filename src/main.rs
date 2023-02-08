use chrono::{DateTime, Local, Timelike};
use std::{process::Command, time};

use std::thread;
fn main() {
    let plug = SmartPlug::new(String::from("10.0.0.44"));
    println!("Amps at startup: {}", plug.get_amp());
    loop {
        run_loop(&plug);
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
fn run_loop(p: &SmartPlug) {
    let mut flag = true;
    let local: DateTime<Local> = Local::now();
    let today = local.format("%A").to_string();
    let mut timer = 0;
    let wait_for = if today == "Saturday" || today == "Sunday" {
        10800
    } else {
        3600
    };
    println!("Waiting for {} hrs", wait_for / 3600);
    while flag {
        if p.get_amp() > 0.5 {
            println!("Setting Time, timer is at {}", timer);
            thread::sleep(time::Duration::from_secs(60));
            timer += 60;
        } else if p.get_amp() == 0.0 {
            println!("TV is off,checking if 6 am");
            let local: DateTime<Local> = Local::now();
            let hrs = local.hour();
            if hrs == 6 {
                p.on()
            }
            thread::sleep(time::Duration::from_secs(10));
        } else {
            println!("TV is idle, timer is at {}", timer);
            thread::sleep(time::Duration::from_secs(60));
        }
        if timer > 3600 {
            flag = false;
            p.off();
        }
    }
    println!("The value has been true for {} secs", timer);
}
pub struct SmartPlug {
    host: String,
}
impl SmartPlug {
    pub fn new(host_ip: String) -> Self {
        Self { host: host_ip }
    }
    fn get_amp(&self) -> f32 {
        let out = Command::new("kasa")
            .arg("--host")
            .arg(&self.host)
            .arg("emeter")
            .output()
            .expect("kasa command failed to start");
        let rst = String::from_utf8(out.stdout).unwrap();
        let split = rst.split('\n');
        let vec: Vec<&str> = split.collect();
        let cur_str = vec[2];
        let val: Vec<&str> = cur_str.split_whitespace().collect();
        let cur = val[1].parse::<f32>();
        cur.unwrap_or(0.0)
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

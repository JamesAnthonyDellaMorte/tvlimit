use chrono::{DateTime, Local, Timelike};
use std::time;
use std::{fs, thread};
mod smart_plug;
fn main() {
    let mut plug = smart_plug::SmartPlug::new(String::from("10.0.0.44:9999"));
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
fn run_loop(p: &mut smart_plug::SmartPlug) {
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
    if timer != 0 {
        println!("Starting timer at {} per text file", timer);
    }
    while flag {
        match p.state {
            smart_plug::PlugState::On => {
                if (timer % 60) == 0 {
                    println!("TV is on! Timer is at {} mins", timer / 60);
                    fs::write("tvtimer.txt", timer.to_string()).unwrap_or(());
                }
                thread::sleep(time::Duration::from_secs(1));
                timer += 1;
                p.update_state();
            }
            smart_plug::PlugState::Off => {
                println!("TV is off,checking if 6 am");
                while smart_plug::PlugState::Off == p.state {
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
            smart_plug::PlugState::Idle => {
                println!("TV is idle, timer is at {} secs", timer);
                while smart_plug::PlugState::Idle == p.state {
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
            smart_plug::PlugState::Unknown => {
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

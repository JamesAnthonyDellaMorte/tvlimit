use chrono::{DateTime, Local, Timelike};
use std::sync::mpsc;
use std::time;
use std::{fs, thread};
mod smart_plug;
fn main() {
    let mut plug = smart_plug::SmartPlug::new(String::from("10.0.0.44:9999"));
    let (send, recv) = mpsc::channel();
    println!("Amps at startup: {} A", plug.get_amps());
    thread::spawn(move || loop {
        let is_six = check_if_6();
        send.send(is_six).unwrap();
        thread::sleep(time::Duration::from_secs(5));
    });
    loop {
        run_loop(&mut plug, &recv);
        time_till_6();
        while !recv.recv().unwrap() {
            thread::sleep(time::Duration::from_secs(10));
        }
        println!("Its 6 am!");
        plug.on();
    }
}
fn check_if_6() -> bool {
    let local: DateTime<Local> = Local::now();
    let hrs = local.hour();
    let mins = local.minute();
    hrs == 6 && mins == 0
}
fn time_till_6() {
    let now = Local::now();
    let target_time = now
        .with_hour(6)
        .unwrap()
        .with_minute(0)
        .unwrap()
        .with_second(0)
        .unwrap();
    let sleep_duration = target_time.signed_duration_since(now) + chrono::Duration::hours(24);
    let total_seconds = sleep_duration.num_seconds();

    let (hours, remainder) = (total_seconds / 3600, total_seconds % 3600);
    let (minutes, seconds) = (remainder / 60, remainder % 60);

    println!("We are waiting until 6:00 to turn on TV! There are {hours} hours, {minutes} minutes, and {seconds} seconds left");
}
fn run_loop(p: &mut smart_plug::SmartPlug, r: &std::sync::mpsc::Receiver<bool>) {
    let mut time_up = false;

    let local: DateTime<Local> = Local::now();
    let mut today = local.format("%A").to_string();
    let mut timer = fs::read_to_string("tvtimer.txt")
        .unwrap_or_else(|_| "0\n".to_owned())
        .trim()
        .parse::<i32>()
        .unwrap_or(0);
    let mut wait_for = if today == "Saturday" || today == "Sunday" {
        7200
    } else {
        3600
    };

    println!("Waiting for {} hrs", wait_for / 3600);
    if timer != 0 {
        println!("Starting timer at {timer} per text file");
    }
    while !time_up {
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
                    let is_6 = r.recv().unwrap();
                    if is_6 {
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
                println!("TV is idle, timer is at {timer} secs");
                while smart_plug::PlugState::Idle == p.state {
                    thread::sleep(time::Duration::from_secs(1));
                    let is_6 = r.recv().unwrap();
                    if is_6 {
                        println!("A new day without all TV time being used!");
                        thread::sleep(time::Duration::from_secs(10));
                        timer = 0;
                        fs::write("tvtimer.txt", timer.to_string()).unwrap_or(());
                        today = local.format("%A").to_string();
                        wait_for = if today == "Saturday" || today == "Sunday" {
                            7200
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
            time_up = true;
            timer = 0;
            fs::write("tvtimer.txt", timer.to_string()).unwrap_or(());
            p.off();
        }
    }
    println!("The value has been true for {timer} secs");
}

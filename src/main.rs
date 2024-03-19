use chrono::{DateTime, Local, Timelike};
use std::time;
use std::{fs, thread};
mod smart_plug;
fn main() {
    let mut plug = smart_plug::SmartPlug::new(String::from("10.0.0.44:9999"));
    println!("Amps at startup: {} A", plug.get_amps());
    loop {
        run_loop(&mut plug);
        wait_till_1230();
        plug.on();
    }
}

fn wait_till_1230() {
    let now = Local::now();
    let target_time = now.with_hour(12)
        .and_then(|time| time.with_minute(30))
        .and_then(|time| time.with_second(0));

    match target_time {
        Some(target_time) => {
            let sleep_duration = if target_time > now {
                target_time - now
            } else {
                target_time + time::Duration::from_secs(86400) - now
            };
            let total_seconds = sleep_duration.num_seconds();
            let (hours, remainder) = (total_seconds / 3600, total_seconds % 3600);
            let (minutes, seconds) = (remainder / 60, remainder % 60);

            println!("We are waiting until 12:30 pm to turn on TV! There are {} hours, {} minutes, and {} seconds left.", hours, minutes, seconds);

            thread::sleep(time::Duration::from_secs(total_seconds as u64));
            println!("It's 12:30 pm!");
        },
        None => eprintln!("Failed to set target time."),
    }
}

fn run_loop(p: &mut smart_plug::SmartPlug) {
    let mut can_watch_tv = true;
    let local: DateTime<Local> = Local::now();
    let today = local.format("%A").to_string();
    let mut timer = fs::read_to_string("tvtimer.txt")
        .unwrap_or("0\n".to_owned())
        .trim()
        .parse::<i32>()
        .unwrap_or(0);
    let wait_for = if today == "Saturday" || today == "Sunday" {
        7200
    } else {
        5400
    };
    let hours = wait_for / 3600;
    let minutes = (wait_for % 3600) / 60;
    if minutes > 0 {
        println!("Waiting for {} hrs and {} mins", hours, minutes);
    } else {
        println!("Waiting for {} hrs", hours);
    }
    if timer != 0 {
        println!("Starting timer at {timer} per text file");
    }
    p.update_state();
    while can_watch_tv {
        match p.state {
            smart_plug::TVState::Play => {
                if (timer % 30) == 0 {
                    println!(
                        "TV is Playing! Timer is at {} mins, Looks like something on {} is playing",
                        (timer as f32) / 60.0,
                        p.whats_playing
                    );
                    fs::write("tvtimer.txt", timer.to_string()).unwrap_or(());
                }
                thread::sleep(time::Duration::from_secs(5));
                timer += 5;
                p.update_state();
                if smart_plug::TVState::Play != p.state {
                    println!("State Change from Play! State is now {:?}", p.state);
                }
            }
            smart_plug::TVState::On => {
                println!("TV is On! But nothing is playing");
                while smart_plug::TVState::On == p.state {
                    thread::sleep(time::Duration::from_secs(5));
                    p.update_state();
                }
                println!("State Change from On! State is now {:?}", p.state);
            }
            smart_plug::TVState::Off => {
                println!("TV is off");
                while smart_plug::TVState::Off == p.state {
                    let local: DateTime<Local> = Local::now();
                    let hrs = local.hour();
                    if hrs == 21 || hrs == 22 || hrs == 23 {
                        can_watch_tv = false;
                        timer = 0;
                        fs::write("tvtimer.txt", timer.to_string()).unwrap_or(());
                        break;
                    }
                    thread::sleep(time::Duration::from_secs(5));
                    p.update_state();
                }
                println!("State Change from Off! State is now {:?}", p.state);
            }
            smart_plug::TVState::Idle => {
                println!("TV is idle, timer is at {timer} secs");
                while smart_plug::TVState::Idle == p.state {
                    thread::sleep(time::Duration::from_secs(5));
                    let hrs = local.hour();
                    if hrs == 21 {
                        can_watch_tv = false;
                        timer = 0;
                        fs::write("tvtimer.txt", timer.to_string()).unwrap_or(());
                        break;
                    }
                    p.update_state();
                }
                println!("State Change from Idle! State is now {:?}", p.state);
            }
            smart_plug::TVState::Unknown => {
                p.update_state();
            }
        }
        if timer > wait_for {
            can_watch_tv = false;
            println!("The value has been true for {timer} secs");
            timer = 0;
            fs::write("tvtimer.txt", timer.to_string()).unwrap_or(());
            p.off();
        }
    }
}

#![recursion_limit = "1024"]

#[macro_use] extern crate error_chain;
#[macro_use] extern crate log;
extern crate env_logger;
extern crate clap;

mod errors;
mod cpu;
mod hdd;
mod sensors;

use std::thread;
use std::env;
use std::time::Duration;
use log::LevelFilter;
use clap::App;
use nix::sys::signal::*;
use errors::*;
use cpu::*;
use hdd::*;
use sensors::*;

const DEFAULT_INTERVAL: u64 = 3;
static mut QUIT: bool = false;

#[derive(PartialEq, Debug)]
struct PemonEntry {
    cpu_info: Vec<CpuInfoEntry>,
    sensor: Sensor,
    hdd_temp: usize,
}

extern "C" fn terminate(_: nix::libc::c_int)
{
    unsafe { QUIT = true; }
}

fn register_signals() -> Result<()> {
    let act = SigAction::new(
        SigHandler::Handler(terminate),
        SaFlags::empty(),
        SigSet::empty(),
    );
    unsafe { sigaction(SIGINT, &act) }.chain_err(|| "Register SIGINT failed.")?;
    unsafe { sigaction(SIGTERM, &act) }.chain_err(|| "Register SIGTERM failed.")?;
    Ok(())
}

fn collect(cpu_stats: &mut Vec<CpuStat>) -> Result<PemonEntry> {
    let cpu_info = collect_cpu_info(cpu_stats)?;
    let sensor = get_sensor_info()?;
    let hdd_temp = get_nvme_hdd_temp()?;
    Ok(PemonEntry {
        cpu_info: cpu_info,
        sensor: sensor,
        hdd_temp: hdd_temp,
    })
}

fn main() {
    env_logger::init();
    log::set_max_level(LevelFilter::Debug);

    let mut itv = DEFAULT_INTERVAL;
    let matches = App::new("pemon")
                        .version("0.1.0")
                        .author("Mark Zhang <ace119@163.com>")
                        .about("A simple utility to collect frequencies and temperatures.")
                        .args_from_usage("-i, --interval=[seconds] 'Seconds delayed before next collection, default: 3 seconds'")
                        .get_matches();

    if let Some(s) = matches.value_of("interval") {
        if let Ok(us) = s.parse::<u64>() {
            itv = us;
        }
    }

    let user = env::var("USER").unwrap();
    debug!("user is: {}", user);
    if user != "root" {
        error!("Permission denied: in order to get some HW info(like HDD temperature), you must run this program as root.");
        return;
    }

    info!("Pemon starts running...");
    match register_signals() {
        Ok(_) => (),
        Err(e) => {
            for t in e.iter() { error!("Register signal failed: {}", t); }
            return;
        },
    }

    let cpu_num = match get_cpu_num() {
        Ok(o) => o,
        Err(e) => {
            for t in e.iter() { error!("Get CPU number failed: {}", t); }
            return;
        },
    };
    info!("CPU number: {}", cpu_num);

    info!("Initialize CPU stats...");
    let mut cpu_stats = match initial_cpu_stats(cpu_num) {
        Ok(o) => o,
        Err(e) => {
            for t in e.iter() { error!("Initial cpu stats failed: {}", t); }
            return;
        },
    };
    thread::sleep(Duration::from_secs(itv));

    let mut pemon = Vec::new();
    loop {
        let entry = match collect(&mut cpu_stats) {
            Ok(o) => o,
            Err(e) => {
                for t in e.iter() { error!("Collect performance info failed: {}", t); }
                break;
            },
        };
        pemon.push(entry);

        unsafe {
            if QUIT {
                info!("Pemon is terminating...");
                break;
            }
        }

        thread::sleep(Duration::from_secs(itv));
    }
}

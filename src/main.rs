#![recursion_limit = "1024"]

#[macro_use] extern crate error_chain;
#[macro_use] extern crate log;
extern crate env_logger;
extern crate clap;

mod errors;

use std::fs::File;
use std::thread;
use std::time::Duration;
use std::io::prelude::*;
use log::LevelFilter;
use clap::App;
use nix::sys::signal::*;
use errors::*;

const DEFAULT_INTERVAL: u64 = 2;
const CPU_FREQ_FILE:&'static str = "/proc/cpuinfo";
static mut QUIT: bool = false;

#[derive(PartialEq, Debug)]
struct CpuFreqEntry {
    id: u32,
    freq: f64,
}

#[derive(PartialEq, Debug)]
struct FanEntry {
    id: u32,
    rpm: u32,
}

#[derive(PartialEq, Debug)]
struct PemonEntry {
    cpu_freqs: Vec<CpuFreqEntry>,
    cpu_temp: u32,
    fans: Vec<FanEntry>,
    hdd_temp: u32,
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

fn collect() -> Result<PemonEntry> {
    let mut f = File::open(CPU_FREQ_FILE)?;
    let mut contents = String::new();
    f.read_to_string(&mut contents)?;

    let mut ret = PemonEntry {
        cpu_freqs: Vec::new(),
        cpu_temp: 0,
        fans: Vec::new(),
        hdd_temp: 0,
    };
    Ok(ret)
}

fn main() {
    env_logger::init();
    log::set_max_level(LevelFilter::Debug);

    let mut itv = DEFAULT_INTERVAL;
    let matches = App::new("pemon")
                        .version("0.1.0")
                        .author("Mark Zhang <ace119@163.com>")
                        .about("A simple utility to collect frequencies and temperatures.")
                        .args_from_usage("-i, --interval=[seconds] 'Seconds delayed before next collection, default: 2 seconds'")
                        .get_matches();

    if let Some(s) = matches.value_of("interval") {
        if let Ok(us) = s.parse::<u64>() {
            itv = us;
        }
    }

    info!("Pemon starts running...");
    match register_signals() {
        Ok(_) => (),
        Err(e) => {
            for t in e.iter() { error!("Register signal failed: {}", t); }
            return;
        },
    }

    loop {
        let entry = match collect() {
            Ok(o) => o,
            Err(e) => {
                for t in e.iter() { error!("Collect CPU frequency info failed: {}", t); }
                break;
            },
        };

        unsafe {
            if QUIT {
                info!("Pemon is terminating...");
                break;
            }
        }

        thread::sleep(Duration::from_secs(itv));
    }
}

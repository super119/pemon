use std::io::prelude::*;
use std::fs::File;
use crate::errors::*;

const CPU_FREQ_FILE:&'static str = "/proc/cpuinfo";
const CPU_STAT_FILE:&'static str = "/proc/stat";

#[derive(PartialEq, Debug)]
pub struct CpuStat {
    user: usize,
    nice: usize,
    system: usize,
    idle: usize,
    iowait: usize,
    irq: usize,
    softirq: usize,
    stealstolen: usize,
    guest: usize,
}

impl CpuStat {
    pub fn new() -> CpuStat {
        CpuStat {
            user: 0,
            nice: 0,
            system: 0,
            idle: 0,
            iowait: 0,
            irq: 0,
            softirq: 0,
            stealstolen: 0,
            guest: 0,
        }
    }
}

#[derive(PartialEq, Debug)]
pub struct CpuInfoEntry {
    id: u32,
    freq: f64,
    usage: f64,
}

pub fn collect_cpu_info(cpu_stats: &mut Vec<CpuStat>) -> Result<Vec<CpuInfoEntry>> {
    let mut result = Vec::new();
    let mut f = File::open(CPU_FREQ_FILE)?;
    let mut contents = String::new();
    f.read_to_string(&mut contents)?;

    let mut id = 1;
    for l in contents.lines() {
        let line = l.trim().to_string();
        if line.len() == 0 {
            continue;
        }

        if line.starts_with("cpu MHz") {
            if let Some(pos) = line.find(':') {
                let sfreq = line[(pos + 1)..].trim().to_string();
                if let Ok(freq) = sfreq.parse::<f64>() {
                    let cfe = CpuInfoEntry {
                        id: id,
                        freq: freq,
                        usage: 0.0,
                    };
                    result.push(cfe);
                    id += 1;
                } else {
                    warn!("Illegal cpuinfo line is found: {}", line);
                    bail!(ErrorKind::InvalidCpuFreqLine);
                }
            } else {
                warn!("Illegal cpuinfo line is found: {}", line);
                bail!(ErrorKind::InvalidCpuFreqLine);
            }
        }
    }

    let mut f = File::open(CPU_STAT_FILE)?;
    let mut contents = String::new();
    f.read_to_string(&mut contents)?;
    for i in 0..result.len() {
        let name = format!("cpu{}", i);
        let mut cs = CpuStat::new();
        for ref l in contents.lines() {
            let line = l.trim().to_string();
            if line.len() == 0 {
                continue;
            }

            if line.starts_with(name.as_str()) {
                let v: Vec<&str> = line[5..].split(' ').collect();
                let user = v[0].parse::<usize>().unwrap();
                let nice = v[1].parse::<usize>().unwrap();
                let system = v[2].parse::<usize>().unwrap();
                let idle = v[3].parse::<usize>().unwrap();
                let iowait = v[4].parse::<usize>().unwrap();
                let irq = v[5].parse::<usize>().unwrap();
                let softirq = v[6].parse::<usize>().unwrap();
                let stealstolen = v[7].parse::<usize>().unwrap();
                let guest = v[8].parse::<usize>().unwrap();
                cs = CpuStat {
                    user: user,
                    nice: nice,
                    system: system,
                    idle: idle,
                    iowait: iowait,
                    irq: irq,
                    softirq: softirq,
                    stealstolen: stealstolen,
                    guest: guest,
                };
                break;
            }
        }

        if cs.user == 0 {
            bail!(ErrorKind::CpuStatNotFound);
        }
        let new_stat = cs;
        let old_stat = &cpu_stats[i];
        let total = (new_stat.user + new_stat.nice + new_stat.system + new_stat.idle +
                    new_stat.iowait + new_stat.irq + new_stat.softirq + new_stat.stealstolen +
                    new_stat.guest) - (old_stat.user + old_stat.nice + old_stat.system +
                    old_stat.idle + old_stat.iowait + old_stat.irq + old_stat.softirq +
                    old_stat.stealstolen + old_stat.guest);
        let idle = new_stat.idle - old_stat.idle;
        result[i].usage = 100.0 * (total - idle) as f64 / total as f64;
        cpu_stats[i] = new_stat;
    }

    Ok(result)
}

pub fn get_cpu_num() -> Result<usize> {
    let mut f = File::open(CPU_FREQ_FILE)?;
    let mut contents = String::new();
    f.read_to_string(&mut contents)?;

    let mut count = 0;
    for l in contents.lines() {
        let line = l.trim().to_string();
        if line.len() == 0 {
            continue;
        }

        if line.starts_with("processor") {
            count += 1;
        }
    }
    Ok(count)
}

pub fn initial_cpu_stats(cpu_num: usize) -> Result<Vec<CpuStat>> {
    let mut result: Vec<CpuStat> = Vec::new();
    let mut f = File::open(CPU_STAT_FILE)?;
    let mut contents = String::new();
    f.read_to_string(&mut contents)?;

    for i in 0..cpu_num {
        let name = format!("cpu{}", i);
        let mut cs = CpuStat::new();
        for ref l in contents.lines() {
            let line = l.trim().to_string();
            if line.len() == 0 {
                continue;
            }

            if line.starts_with(name.as_str()) {
                let v: Vec<&str> = line[5..].split(' ').collect();
                let user = v[0].parse::<usize>().unwrap();
                let nice = v[1].parse::<usize>().unwrap();
                let system = v[2].parse::<usize>().unwrap();
                let idle = v[3].parse::<usize>().unwrap();
                let iowait = v[4].parse::<usize>().unwrap();
                let irq = v[5].parse::<usize>().unwrap();
                let softirq = v[6].parse::<usize>().unwrap();
                let stealstolen = v[7].parse::<usize>().unwrap();
                let guest = v[8].parse::<usize>().unwrap();

                cs = CpuStat {
                    user: user,
                    nice: nice,
                    system: system,
                    idle: idle,
                    iowait: iowait,
                    irq: irq,
                    softirq: softirq,
                    stealstolen: stealstolen,
                    guest: guest,
                };
                break;
            }
        }

        if cs.user == 0 {
            bail!(ErrorKind::CpuStatNotFound);
        } else {
            result.push(cs);
        }
    }
    Ok(result)
}

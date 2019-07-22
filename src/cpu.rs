use std::io::prelude::*;
use std::fs::File;
use crate::errors::*;

const CPU_FREQ_FILE:&'static str = "/proc/cpuinfo";
const CPU_STAT_FILE:&'static str = "/proc/stat";

#[derive(PartialEq, Debug)]
pub struct CpuStat {
    total: usize,
    idle: usize,
}

impl CpuStat {
    fn new() -> CpuStat {
        CpuStat {
            total: 0,
            idle: 0,
        }
    }
}

#[derive(PartialEq, Debug)]
pub struct CpuInfoEntry {
    pub id: usize,
    pub freq: f64,
    pub usage: f64,
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
                let v: Vec<&str> = line[(name.len() + 1)..].split(' ').collect();
                let mut total = 0;
                for i in 0..v.len() {
                    total += v[i].parse::<usize>().unwrap();
                }
                // The 4th member is idle jiffies, kernel won't break userspace
                let idle = v[3].parse::<usize>().unwrap();

                cs = CpuStat {
                    total: total,
                    idle: idle,
                };
                break;
            }
        }

        if cs.total == 0 {
            bail!(ErrorKind::CpuStatNotFound);
        }
        let new_stat = cs;
        let old_stat = &cpu_stats[i];
        let total = new_stat.total - old_stat.total;
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
                let v: Vec<&str> = line[(name.len() + 1)..].split(' ').collect();
                let mut total = 0;
                for i in 0..v.len() {
                    total += v[i].parse::<usize>().unwrap();
                }
                let idle = v[3].parse::<usize>().unwrap();

                cs = CpuStat {
                    total: total,
                    idle: idle,
                };
                break;
            }
        }

        if cs.total == 0 {
            bail!(ErrorKind::CpuStatNotFound);
        } else {
            result.push(cs);
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_get_cpu_num() {
        let result = get_cpu_num().unwrap();
        assert_eq!(result, 16);
    }

    #[test]
    fn test_collect_cpu_info() {
        let mut stats = initial_cpu_stats(get_cpu_num().unwrap()).unwrap();
        thread::sleep(std::time::Duration::from_secs(2));
        let result = collect_cpu_info(&mut stats).unwrap();
        for i in 0..result.len() {
            let cie = &result[i];
            assert_eq!(cie.id, i+1);
            assert_eq!(cie.freq > 0.0, true );
            assert_eq!(cie.usage >= 0.0, true );
            println!("CPU {} freq: {}, usage: {}", cie.id, cie.freq, cie.usage);
        }
    }
}

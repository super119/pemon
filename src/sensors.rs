use std::process::Command;
use std::os::unix::process::CommandExt;
use crate::errors::*;
use nix::sys::signal::*;

#[derive(PartialEq, Debug)]
pub struct Sensor {
    cpu_temp: usize,
    mb_temp: usize,
    chipset_temp: usize,
    cpu_fan_rpm: usize,
    chassis_fan_rpm: usize,
}

fn get_temperature(line: &String) -> Result<usize> {
    let result;
    if let Some(pos) = line.find(':') {
        let temp_str = line[(pos + 1)..].trim();
        if let Some(pos) = temp_str.find('.') {
            // 1 means to skip the '+' before the temperature
            let temp = temp_str[1..pos].parse::<usize>()?;
            result = temp;
        } else {
            warn!("Parsing sensors output failed, illegal temperature line: {}", line);
            bail!(ErrorKind::GetTempFailed);
        }
    } else {
        warn!("Parsing sensors output failed, illegal temperature line: {}", line);
        bail!(ErrorKind::GetTempFailed);
    }
    Ok(result)
}

fn get_rpm(line: &String) -> Result<usize> {
    let result;
    if let Some(pos) = line.find(':') {
        let temp_str = line[(pos + 1)..].trim();
        if let Some(pos) = temp_str.find(' ') {
            let rpm = temp_str[0..pos].parse::<usize>()?;
            result = rpm;
        } else {
            warn!("Parsing sensors output failed, illegal rpm line: {}", line);
            bail!(ErrorKind::GetRpmFailed);
        }
    } else {
        warn!("Parsing sensors output failed, illegal rpm line: {}", line);
        bail!(ErrorKind::GetRpmFailed);
    }
    Ok(result)
}

pub fn get_sensor_info() -> Result<Sensor> {
    let mut cpu = 0;
    let mut mb = 0;
    let mut chipset = 0;
    let mut cpu_fan = 0;
    let mut cha_fan = 0;
    let output;
    unsafe {
        output = Command::new("sensors")
                 // pre_exec is unsafe function
                 .pre_exec(|| {
                     let mut set = SigSet::empty();
                     set.add(SIGINT);
                     set.add(SIGTERM);
                     sigprocmask(SigmaskHow::SIG_BLOCK, Some(&set), None).unwrap();
                     Ok(())
                 })
                 .output()?;
    }
    if !output.status.success() {
        bail!("Running sensors failed. Sensor info is unavailable now.");
    }

    let out = String::from_utf8_lossy(&output.stdout).into_owned();
    for l in out.lines() {
        let line = l.trim().to_string();
        if line.len() == 0 {
            continue;
        }

        if line.starts_with("CPU Temperature") {
            cpu = get_temperature(&line)?;
        }
        if line.starts_with("Motherboard Temperature") {
            mb = get_temperature(&line)?;
        }
        if line.starts_with("Chipset Temperature") {
            chipset = get_temperature(&line)?;
        }
        if line.starts_with("CPU Fan") {
            cpu_fan = get_rpm(&line)?;
        }
        if line.starts_with("Chassis Fan 1") {
            cha_fan = get_rpm(&line)?;
        }
    }
    Ok(Sensor {
        cpu_temp: cpu,
        mb_temp: mb,
        chipset_temp: chipset,
        cpu_fan_rpm: cpu_fan,
        chassis_fan_rpm: cha_fan,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_sensor() {
        let result = get_sensor_info().unwrap();
        assert_eq!(result.cpu_temp > 0, true);
        println!("Sensor: cpu temp: {}, mb temp: {}, chipset temp: {}, cpu fan: {} RPM, chassis fan: {} RPM",
                 result.cpu_temp, result.mb_temp, result.chipset_temp, result.cpu_fan_rpm, result.chassis_fan_rpm);
    }
}

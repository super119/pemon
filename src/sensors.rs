extern crate subprocess;

use crate::errors::*;
use subprocess::Exec;
use subprocess::Redirection;

pub fn get_cpu_temp() -> Result<usize> {
    let mut result = 0;
    let mut found = false;
    let p = Exec::cmd("sensors")
              .stdout(Redirection::Pipe)
              .capture()?;
    if !p.success() {
        bail!("Running sensors failed. CPU temperature is unavailable.");
    }

    let out = p.stdout_str();
    for l in out.lines() {
        let line = l.trim().to_string();
        if line.len() == 0 {
            continue;
        }

        if line.starts_with("temp1") {
            if let Some(pos) = line.find(':') {
                let temp_str = line[(pos + 1)..].trim();
                if let Some(pos) = temp_str.find('.') {
                    // 1 means to skip the '+' before the temperature
                    let temp = temp_str[1..pos].parse::<usize>().unwrap();
                    result = temp;
                    found = true;
                    break;
                } else {
                    warn!("Parsing sensors output failed, illegal temperature line: {}", line);
                }
            } else {
                warn!("Parsing sensors output failed, illegal temperature line: {}", line);
            }
        }
    }

    if !found {
        warn!("Getting CPU temperature failed.");
        bail!(ErrorKind::GetCpuTempFailed);
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_cpu_temp() {
        let result = get_cpu_temp().unwrap();
        assert_eq!(result > 0, true);
        println!("Got CPU temperature: {}", result);
    }
}

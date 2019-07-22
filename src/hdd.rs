use std::process::Command;
use std::os::unix::process::CommandExt;
use crate::errors::*;
use nix::sys::signal::*;

pub fn get_nvme_hdd_temp() -> Result<usize> {
    let mut result = 0;
    let mut found = false;
    let output;
    unsafe {
        output = Command::new("nvme")
                 .arg("smart-log")
                 .arg("/dev/nvme0n1")
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
        bail!("Running nvme failed. HDD temp is unavailable.");
    }
    let out = String::from_utf8_lossy(&output.stdout).into_owned();
    for l in out.lines() {
        let line = l.trim().to_string();
        if line.len() == 0 {
            continue;
        }

        if line.starts_with("temperature") {
            if let Some(pos) = line.find(':') {
                let temp_str = line[(pos + 1)..].trim();
                if let Some(pos) = temp_str.find(' ') {
                    let temp = temp_str[0..pos].parse::<usize>().unwrap();
                    result = temp;
                    found = true;
                    break;
                } else {
                    warn!("Parsing nvme output failed, illegal temperature line: {}", line);
                }
            } else {
                warn!("Parsing nvme output failed, illegal temperature line: {}", line);
            }
        }
    }

    if !found {
        warn!("Getting nvme HDD temperature failed.");
        bail!(ErrorKind::GetNvmeHDDTempFailed);
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_nvme_hdd_temp() {
       let result = get_nvme_hdd_temp().unwrap();
       assert_eq!(result > 0, true);
       println!("Got HDD temperature: {}", result);
    }
}

use crate::common;
use crate::common::Options;

use std::error::Error;
use std::path::Path;
use std::process::Command;

pub fn execute_cmd(cmd: String) -> Result<(), Box<Error>> {
    let cmd_split: Vec<_> = cmd.split(' ').collect();
    if cmd_split.len() < 1 || cmd_split[0] == "" {
        return Err("No command specified")?;
    }

    let proc;

    if cmd_split.len() < 2 {
        proc = Command::new(cmd_split[0]).spawn()?;
    } else {
        proc = Command::new(cmd_split[0])
            .args(&cmd_split[1..cmd_split.len()])
            .spawn()?;
    }

    let _ = proc.wait_with_output()?;

    Ok(())
}

pub fn run_openvpn_command(hostnames: Vec<&str>, opt: &Options) -> Result<(), Box<Error>> {
    let v = opt.verbose;
    let command = build_openvpn_command(hostnames, opt);
    common::vprint(v, command.as_str());

    execute_cmd(command)?;

    Ok(())
}

pub fn build_openvpn_command(hostnames: Vec<&str>, opt: &Options) -> String {
    let mut first_host = String::new();
    let mut found = false;
    for hostname in hostnames {
        let hostname = hostname.to_string()
            + if opt.use_udp {
                opt.udp_suffix
            } else {
                opt.tcp_suffix
            };
        let fullpath = Path::new(opt.base_folder).join(Path::new(hostname.as_str()));

        if fullpath.exists() {
            found = true;
            common::vprint(opt.verbose, "Found hostname file");
            first_host = match fullpath.to_str() {
                Some(v) => v.to_string(),
                None => String::new(),
            };
            break;
        }
    }

    if !found {
        common::vprint(opt.verbose, "No hostname file found...");
    }

    let command = format!(
        "sudo openvpn --config {}{}",
        first_host,
        match opt.credential_file {
            Some(v) => match Path::new(v).file_stem() {
                Some(v) => format!(" --auth-user-pass {}", v.to_str().unwrap()),
                None => "".to_string(),
            },
            None => "".to_string(),
        }
    );

    command
}

pub fn unlock_gpg(file_name: &str) -> Result<(), Box<Error>> {
    let command = format!("gpg {}", file_name);
    execute_cmd(command)?;
    Ok(())
}

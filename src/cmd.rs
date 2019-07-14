extern crate rand;

use rand::seq::SliceRandom;

use crate::common;
use crate::common::Options;

use std::error::Error;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

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

pub fn run_openvpn_command(hostnames: &mut Vec<String>, opt: &Options) -> Result<(), Box<Error>> {
    let command = build_openvpn_command(hostnames, opt)?;
    common::vprint(opt.verbose, command.as_str());

    execute_cmd(command)?;

    Ok(())
}

pub fn build_openvpn_command(
    hostnames: &mut Vec<String>,
    opt: &Options,
) -> Result<String, Box<Error>> {
    if opt.random {
        hostnames.shuffle(&mut rand::thread_rng());
    }
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
        Err("No hostname file found from provided")?;
    }

    let command = format!(
        "sudo openvpn --config {}{}",
        first_host,
        match opt.credential_file {
            Some(v) => format!(" --auth-user-pass {}", remove_file_extension(v)),
            None => "".to_string(),
        }
    );

    Ok(command)
}

fn remove_file_extension(file: &str) -> String {
    match Path::new(file).file_stem() {
        Some(v) => v.to_str().unwrap().to_string(),
        None => "".to_string(),
    }
}

pub fn unlock_gpg(file_name: &str, opt: &Options) -> Result<Option<JoinHandle<()>>, Box<Error>> {
    let command = format!("gpg {}", file_name);

    execute_cmd(command)?;

    let mut delete_file_handler = None;
    if let Some(file) = opt.credential_file {
        let file = remove_file_extension(file);
        delete_file_handler = Some(thread::spawn(move || {
            thread::sleep(Duration::from_secs(5));
            fs::remove_file(file).unwrap();
        }));
    }

    Ok(delete_file_handler)
}

extern crate reqwest;
extern crate serde;
extern crate serde_json;

use serde::{Deserialize, Serialize};

use std::error::Error;

use crate::common;
use crate::common::Country;

pub struct Options {
    pub country: Country,
    pub verbose: bool,
}

impl Options {
    fn get_id(&self) -> Option<u32> {
        match self.country {
            Country::Canada => Some(38),
            Country::US => Some(228),
            _ => None,
        }
    }
}

pub struct Servers {
    servers: Vec<Server>,
    options: Options,
    base_url: String,
}

impl Servers {
    pub fn new(url: &str, opt: Options) -> Servers {
        Servers {
            servers: vec![],
            options: opt,
            base_url: url.to_string(),
        }
    }

    pub fn update_server_list(mut self) -> Result<Self, Box<Error>> {
        let mut new_url = self.base_url.clone();
        let country_id = self.options.get_id();
        common::vprint(self.options.verbose, "About to do server list update...");
        if let Some(id) = country_id {
            common::vprint(
                self.options.verbose,
                format!("Adding {} country to request", id).as_str(),
            );
            new_url.push_str(format!("&filters={{\"country_id\":{}}}", id).as_str());
        }
        common::vprint(self.options.verbose, "Starting the request...");
        let resp: String = reqwest::get(new_url.as_str())?.text()?;
        common::vprint(self.options.verbose, "Got results from request");

        let deserialized: Vec<Server> = serde_json::from_str(&resp)?;
        self.servers.extend(deserialized);

        Ok(self)
    }

    pub fn get_servers(&self) -> &Vec<Server> {
        &self.servers
    }

    fn get_best_server(&self) -> Option<&str> {
        if self.servers.len() > 0 {
            Some(self.servers[0].hostname.as_str())
        } else {
            None
        }
    }

    pub fn get_hostnames(&self) -> Vec<String> {
        self.servers.iter().map(|s| s.hostname.clone()).collect()
    }

    pub fn build_command(&self) -> Result<String, Box<Error>> {
        let mut command_string = String::new();
        command_string.push_str("sudo openvpn --config ");
        match self.get_best_server() {
            Some(s) => command_string.push_str(s),
            None => Err("not able to get best server")?,
        }

        Ok(command_string)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Server {
    id: u32,
    created_at: String,
    updated_at: String,
    name: String,
    station: String,
    hostname: String,
    load: u32,
    status: String,
    // locations: Vec<Value>,
    // technologies: Vec<Value>,
    // ips: Vec<Value>,
    // groups: Vec<Value>,
}

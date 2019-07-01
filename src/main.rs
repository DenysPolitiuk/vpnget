extern crate clap;
extern crate crossbeam;
extern crate ctrlc;

use clap::App;
use clap::Arg;

use vpnget::cmd;
use vpnget::common;
use vpnget::common::Country;
use vpnget::common::Options;
use vpnget::ovpn;
use vpnget::Servers;

// TODO:
//
// Get servers
// 1) make a request for best servers +
// 2) optional country +
// 3) add server group and server technology options
//
//  Look into better error handling ?
//  Add logging ?
fn main() {
    const SERVER_URL: &str =
        "https://nordvpn.com/wp-admin/admin-ajax.php?action=servers_recommendations";
    const OVPN_URL: &str = "https://downloads.nordcdn.com/configs/archives/servers/ovpn.zip";
    const OVPN_BASE_FOLDER: &str = "ovpn";
    const UDP_SUF: &str = ".udp.ovpn";
    const DEFAULT_CREDENTIAL: &str = "credentials_vpn.gpg";

    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .arg(
            Arg::with_name("update")
                .short("u")
                .long("update")
                .help("update current ovpn files"),
        )
        .arg(
            Arg::with_name("country")
                .short("c")
                .long("country")
                .takes_value(true)
                .help("specify country to connect to"),
        )
        .arg(
            Arg::with_name("credential")
                .long("auth")
                .takes_value(true)
                .help("use credential file"),
        )
        .arg(
            Arg::with_name("credential_flag")
                .short("a")
                .help("use default credential file"),
        )
        .arg(
            Arg::with_name("force")
                .short("f")
                .long("force")
                .takes_value(true)
                .help("specify a specific ovpn file to use"),
        )
        .arg(
            Arg::with_name("random")
                .short("r")
                .long("random")
                .help("randomize picked server from returned server from request"),
        )
        .arg(
            Arg::with_name("execute")
                .short("e")
                .long("execute")
                .help("execute generated openvpn command"),
        )
        .arg(Arg::with_name("verbose").short("v").help("be verbose"))
        .get_matches();

    let country = matches.value_of("country").unwrap_or("");
    let credential = matches
        .value_of("credential")
        .or(if matches.is_present("credential_flag") {
            Some(DEFAULT_CREDENTIAL)
        } else {
            None
        });
    let execute = matches.is_present("execute");
    let options = Options::new()
        .with_base_folder(OVPN_BASE_FOLDER)
        .with_verbose(matches.is_present("verbose"))
        .with_udp_suffix(UDP_SUF)
        .with_credential_file(credential)
        .with_country(Country::new(country))
        .with_random(matches.is_present("random"))
        .with_forced(matches.value_of("force").or(None));

    let v = options.verbose;
    ctrlc::set_handler(move || {
        common::vprint(v, "There is no escape...");
    })
    .unwrap();

    let mut servers = None;
    crossbeam::scope(|s| {
        if matches.is_present("update") {
            s.spawn(|_| {
                if let Err(e) = ovpn::handler_ovpn_zip(OVPN_URL, &options) {
                    println!("Error from ovpn : {}", e);
                }
            });
        }

        if options.forced.is_none() {
            s.spawn(|_| {
                match Servers::new(
                    SERVER_URL,
                    vpnget::Options {
                        country: Country::new(country),
                        verbose: options.verbose,
                    },
                )
                .update_server_list()
                {
                    Ok(s) => servers = Some(s),
                    Err(e) => println!("Error in update server list : {}", e),
                };
            });
        }
    })
    .unwrap();

    let mut hostnames = match servers {
        Some(s) => s.get_hostnames(),
        None => match options.forced {
            None => panic!("No servers found and no force servers"),
            Some(s) => vec![s.to_string()],
        },
    };

    common::vprint(options.verbose, format!("{:?}", hostnames).as_str());

    if execute {
        let mut delete_file_handler = None;
        if let Some(f) = credential {
            let result = cmd::unlock_gpg(f, &options);
            match result {
                Err(e) => println!("Error from unlock gpg : {}", e),
                Ok(h) => delete_file_handler = h,
            }
        }
        if let Err(e) = cmd::run_openvpn_command(&mut hostnames, &options) {
            println!("Error from cmd : {}", e);
        }
        if let Some(h) = delete_file_handler {
            common::vprint(options.verbose, "Waiting for deleting credential file");
            h.join().unwrap();
            common::vprint(options.verbose, "File deleted");
        }
    } else {
        let command = cmd::build_openvpn_command(&mut hostnames, &options);
        match command {
            Ok(c) => println!("{}", c),
            Err(e) => println!("Error from build command : {}", e),
        };
    }

    common::vprint(options.verbose, "Existing the program");
}

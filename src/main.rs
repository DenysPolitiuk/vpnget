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

// Verbose option
//
// Get servers
// 1) make a request for best servers +
// 2) optional country +
// 3) add server group and server technology options
//
// Run openvpn
// 1) optionally decrypt credential file +
// 2) run openvpn command +
// 3) delete decrypted credential file from #1
// 4) force flag to force a specific server
//
// TODO:
//  Optimize unzip to only unzip specified country
//  Add option to pick random server from returned in request to avoid picking the same server
//      even after re-running in  short period of time
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
        .arg(Arg::with_name("verbose").short("v").help("be verbose"))
        .get_matches();

    let verbose = matches.is_present("verbose");
    let country = matches.value_of("country").unwrap_or("");
    let credential = matches
        .value_of("credential")
        .or(if matches.is_present("credential_flag") {
            Some(DEFAULT_CREDENTIAL)
        } else {
            None
        });
    let options = Options::new()
        .with_base_folder(OVPN_BASE_FOLDER)
        .with_verbose(verbose)
        .with_udp_suffix(UDP_SUF)
        .with_credential_file(credential);
    let force = matches.value_of("force").or(None);
    let random = matches.is_present("random");

    let v = verbose;
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

        s.spawn(|_| {
            match Servers::new(
                SERVER_URL,
                vpnget::Options {
                    country: Country::new(country),
                    verbose,
                },
            )
            .update_server_list()
            {
                Ok(s) => servers = Some(s),
                Err(e) => println!("Error in update server list : {}", e),
            };
        });
    })
    .unwrap();
    let servers = servers.unwrap();

    common::vprint(verbose, format!("{:?}", servers.get_hostnames()).as_str());

    if let Some(f) = credential {
        if let Err(e) = cmd::unlock_gpg(f) {
            println!("Error from unlock gpg : {}", e);
        }
    }

    if let Err(e) = cmd::run_openvpn_command(servers.get_hostnames(), &options) {
        println!("Error from cmd : {}", e);
    }

    common::vprint(verbose, "Existing the program");
}

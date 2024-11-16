use clap::Parser;
use std::env;
use std::path::Path;
use std::fs;

mod checks;
mod pings;
mod config;
mod update;
mod systemd;

fn get_hostname() -> String {
    env::var("HOSTNAME").unwrap_or_else(|_| {
        let output = std::process::Command::new("hostname")
            .output()
            .expect("Failed to execute hostname command");
        String::from_utf8_lossy(&output.stdout).trim().to_string()
    })
}

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Check if updates are available
    #[arg(long)]
    check_update: bool,

    /// Download and install the latest update
    #[arg(long)]
    update: bool,

    /// Show sample systemd service
    #[arg(long)]
    service: bool,
}

fn install_service() -> Result<(), std::io::Error> {
    let service = systemd::SystemdService {
        name: "icinga-passive-checks".to_string(),
        description: "Icinga2 Passive Checks Service".to_string(),
        exec_start: update::running_binary_path().unwrap(),
        after: vec!["network.target".to_string()],
    };

    let unit_file = systemd::generate_unit_content(&service);
    println!("{}", unit_file);

    Ok(())
}

fn read_lsb_release() -> Option<String> {
    if let Ok(content) = fs::read_to_string("/etc/lsb-release") {
        let mut is_ubuntu = false;
        let mut version = None;

        for line in content.lines() {
            if line == "DISTRIB_ID=Ubuntu" {
                is_ubuntu = true;
            }
            if line.starts_with("DISTRIB_RELEASE=") {
                version = line.split('=').nth(1).map(String::from);
            }
        }

        if is_ubuntu {
            version
        } else {
            None
        }
    } else {
        None
    }
}

fn main() {
    let args = Args::parse();
    let config = config::load_config();

    if config.debug {
        println!("Config: {:#?}", config);
    }

    if args.check_update {
        let update_status = update::check_for_updates(env!("CARGO_PKG_VERSION"));
        match update_status {
            Ok(true) => println!("An update is available"),
            Ok(false) => println!("No updates available ({})", env!("CARGO_PKG_VERSION")),
            Err(e) => eprintln!("Error checking for updates: {}", e),
        }
        return;
    }

    if args.update {
        match update::running_binary_path() {
            Ok(path) => {
                let update_status = update::check_for_updates(env!("CARGO_PKG_VERSION")).expect("Error checking for updates");
                let ubuntu_version = read_lsb_release().expect("This program requires Ubuntu");
                let asset_name = format!("icinga_passive_checks.x86_64-ubuntu{}", ubuntu_version);
                let latest_version = update::get_latest_version().expect("Error getting latest version");

                if update_status {
                    match update::download_release_asset(&latest_version, &asset_name, Path::new(&path)) {
                        Ok(_) => println!("Update downloaded successfully"),
                        Err(e) => eprintln!("Error downloading update: {}", e),
                    }
                }
            },
            Err(e) => eprintln!("Error determining binary path: {}", e),
        }
        return;
    }

    if args.service {
        let _ = install_service();
        return;
    }

    // Normal operation
    for ping in &config.pings {
        pings::ping_host(&get_hostname(), &ping.name, &ping.host, &config);
    }
}

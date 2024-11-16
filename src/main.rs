use clap::Parser;
use std::env;
use std::path::Path;

mod checks;
mod pings;
mod config;
mod update;

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
}

fn main() {
    let args = Args::parse();
    let config = config::load_config();

    if config.debug {
        println!("Config: {:#?}", config);
    }

    if args.check_update {
        let update_status = update::check_for_updates(env!("CARGO_PKG_VERSION"));
        println!("Update check: {:?}", update_status);
        return;
    }

    if args.update {
        match update::running_binary_path() {
            Ok(path) => {
                match update::download_release_asset("v0.1.0", "icinga_passive_checks", Path::new(&path)) {
                    Ok(_) => println!("Update downloaded successfully"),
                    Err(e) => eprintln!("Error downloading update: {}", e),
                }
            },
            Err(e) => eprintln!("Error determining binary path: {}", e),
        }
        return;
    }

    // Normal operation
    for ping in &config.pings {
        pings::ping_host(&get_hostname(), &ping.name, &ping.host, &config);
    }
}

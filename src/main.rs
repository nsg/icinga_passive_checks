use std::env;

mod checks;
mod pings;
mod config;

fn get_hostname() -> String {
    env::var("HOSTNAME").unwrap_or_else(|_| {
        let output = std::process::Command::new("hostname")
            .output()
            .expect("Failed to execute hostname command");
        String::from_utf8_lossy(&output.stdout).trim().to_string()
    })
}

fn main() {
    let config = config::load_config();

    if config.debug {
        println!("Config: {:#?}", config);
    }

    for ping in &config.pings {
        pings::ping_host(&get_hostname(), &ping.name, &ping.host, &config);
    }
}

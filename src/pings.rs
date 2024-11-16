use std::process::Command;
use regex::Regex;
use std::collections::HashMap;
use crate::checks::{CheckResult, send_passive_check};
use crate::config::IcingaConfig;

#[derive(Debug)]
struct PingMetrics {
    packet_loss: f64,
    time: i32,
    rtt_min: f64,
    rtt_avg: f64,
    rtt_max: f64,
    rtt_mdev: f64,
}

pub fn ping_host(check_source: &str, check_name: &str, check_host: &str, icinga_config: &IcingaConfig) {
    let ping_data = execute_ping(check_host);
    send_passive_check(
        check_source,
        check_name,
        check_host,
        "Passive Ping",
        &ping_data,
        icinga_config,
    );
}

fn execute_ping(host: &str) -> CheckResult {
    let output = Command::new("ping")
        .arg("-c")
        .arg("8")
        .arg(host)
        .output()
        .expect("Failed to execute ping command");
    let response = String::from_utf8_lossy(&output.stdout);
    let metrics = parse_ping_metrics(&response);
    format_ping_result(&metrics)
}

fn format_ping_result(metrics: &PingMetrics) -> CheckResult {
    let mut result = HashMap::new();
    let status = if metrics.packet_loss == 0.0 { "OK" } else { "CRITICAL" };
    
    result.insert("exit_status".to_string(), 
        if metrics.packet_loss == 0.0 { "0" } else { "2" }.to_string());
    result.insert("plugin_output".to_string(), 
        format!("PING {} - Packet loss = {}% AVG = {}ms", 
            status, metrics.packet_loss, metrics.rtt_avg));
    result.insert("performance_data".to_string(), 
        vec![
            format!("rtavg={}ms;3000;5000;0", metrics.rtt_avg),
            format!("rtmin={}ms;3000;5000;0", metrics.rtt_min),
            format!("rtmax={}ms;3000;5000;0", metrics.rtt_max),
            format!("rtdev={}ms;3000;5000;0", metrics.rtt_mdev),
            format!("pl={}%;80;100;0", metrics.packet_loss),
            format!("time={}ms;8500;10000;0", metrics.time),
        ].join(","));
    
    result
}

fn parse_ping_metrics(response: &str) -> PingMetrics {
    let packet_loss_re = Regex::new(r"(\d+)% packet loss").unwrap();
    let time_re = Regex::new(r"time (\d+)ms").unwrap();
    let rtt_re = Regex::new(r"rtt min/avg/max/mdev = ([\d\.]+)/([\d\.]+)/([\d\.]+)/([\d\.]+) ms").unwrap();

    let packet_loss = match packet_loss_re.captures(response) {
        Some(caps) => caps[1].parse().unwrap_or(100.0),
        None => 100.0,
    };
    let time = match time_re.captures(response) {
        Some(caps) => caps[1].parse().unwrap_or(0),
        None => 0,
    };
    
    let (rtt_min, rtt_avg, rtt_max, rtt_mdev) = match rtt_re.captures(response) {
        Some(caps) => (
            caps[1].parse().unwrap_or(10000.0),
            caps[2].parse().unwrap_or(10000.0),
            caps[3].parse().unwrap_or(10000.0),
            caps[4].parse().unwrap_or(10000.0),
        ),
        None => (10000.0, 10000.0, 10000.0, 10000.0),
    };

    PingMetrics {
        packet_loss,
        time,
        rtt_min,
        rtt_avg,
        rtt_max,
        rtt_mdev,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_ping_success() -> String {
        r#"PING google.com (142.250.74.78) 56(84) bytes of data.
64 bytes from 142.250.74.78: icmp_seq=1 ttl=116 time=4.88 ms
64 bytes from 142.250.74.78: icmp_seq=2 ttl=116 time=4.75 ms
64 bytes from 142.250.74.78: icmp_seq=3 ttl=116 time=4.82 ms
64 bytes from 142.250.74.78: icmp_seq=4 ttl=116 time=4.80 ms

--- google.com ping statistics ---
4 packets transmitted, 4 received, 0% packet loss, time 3004ms
rtt min/avg/max/mdev = 4.752/4.812/4.876/0.047 ms"#.to_string()
    }

    fn mock_ping_failure() -> String {
        r#"PING invalid.host (1.2.3.4) 56(84) bytes of data.

--- invalid.host ping statistics ---
4 packets transmitted, 0 received, 100% packet loss, time 3004ms"#.to_string()
    }

    fn mock_ping_partial_loss() -> String {
        r#"PING partial.host (1.2.3.4) 56(84) bytes of data.
64 bytes from 1.2.3.4: icmp_seq=1 ttl=116 time=4.88 ms
64 bytes from 1.2.3.4: icmp_seq=4 ttl=116 time=4.80 ms

--- partial.host ping statistics ---
4 packets transmitted, 2 received, 50% packet loss, time 3004ms
rtt min/avg/max/mdev = 4.752/4.812/4.876/0.047 ms"#.to_string()
    }

    fn mock_ping_malformed() -> String {
        "Invalid ping output".to_string()
    }

    #[test]
    fn test_parse_ping_metrics() {
        let metrics = parse_ping_metrics(&mock_ping_success());
        assert_eq!(metrics.packet_loss, 0.0);
        assert!(metrics.rtt_avg > 0.0);
    }

    #[test]
    fn test_ping_success() {
        let result = format_ping_result(&parse_ping_metrics(&mock_ping_success()));
        assert_eq!(result.get("exit_status").unwrap(), "0");
        assert!(result.get("plugin_output").unwrap().contains("PING OK"));
        assert!(result.get("performance_data").unwrap().contains("pl=0%"));
    }

    #[test]
    fn test_ping_failure() {
        let result = format_ping_result(&parse_ping_metrics(&mock_ping_failure()));
        assert_eq!(result.get("exit_status").unwrap(), "2");
        assert!(result.get("plugin_output").unwrap().contains("PING CRITICAL"));
        assert!(result.get("performance_data").unwrap().contains("pl=100%"));
    }

    #[test]
    fn test_parse_ping_metrics_success() {
        let metrics = parse_ping_metrics(&mock_ping_success());
        assert_eq!(metrics.packet_loss, 0.0);
        assert_eq!(metrics.time, 3004);
        assert_eq!(metrics.rtt_min, 4.752);
        assert_eq!(metrics.rtt_avg, 4.812);
        assert_eq!(metrics.rtt_max, 4.876);
        assert_eq!(metrics.rtt_mdev, 0.047);
    }

    #[test]
    fn test_parse_ping_metrics_failure() {
        let metrics = parse_ping_metrics(&mock_ping_failure());
        assert_eq!(metrics.packet_loss, 100.0);
        assert_eq!(metrics.rtt_min, 10000.0);
        assert_eq!(metrics.time, 3004);
    }

    #[test]
    fn test_parse_ping_metrics_partial_loss() {
        let metrics = parse_ping_metrics(&mock_ping_partial_loss());
        assert_eq!(metrics.packet_loss, 50.0);
        assert!(metrics.rtt_avg > 0.0);
    }

    #[test]
    fn test_parse_ping_metrics_malformed() {
        let metrics = parse_ping_metrics(&mock_ping_malformed());
        assert_eq!(metrics.packet_loss, 100.0);
        assert_eq!(metrics.time, 0);
        assert_eq!(metrics.rtt_min, 10000.0);
    }
}

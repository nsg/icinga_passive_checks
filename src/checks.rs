use crate::config::IcingaConfig;
use reqwest::{blocking::Client, header::ACCEPT, StatusCode};
use serde_json::Value;
use std::collections::HashMap;

pub type CheckResult = HashMap<String, String>;

const DEFAULT_EXIT_STATUS: i32 = 3; // UNKNOWN status code

type CheckPayload = HashMap<String, Value>;

fn parse_performance_data(perf_data: Option<&String>) -> Vec<String> {
    perf_data
        .map(|data| data.split(',').map(|s| s.trim().to_string()).collect())
        .unwrap_or_default()
}

fn format_check_payload(
    check_source: &str,
    check_type: &str,
    check_name: &str,
    check_data: &CheckResult,
) -> CheckPayload {
    let filter_value = format!(
        "host.name==\"{}\" && service.name==\"{}: {}\"",
        check_source, check_type, check_name
    );

    let exit_status = check_data
        .get("exit_status")
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(DEFAULT_EXIT_STATUS);

    let plugin_output = check_data
        .get("plugin_output")
        .cloned()
        .unwrap_or_else(|| "No output provided".to_string());

    let perf_data = parse_performance_data(check_data.get("performance_data"));

    HashMap::from([
        ("type".to_string(), Value::String("Service".to_string())),
        ("filter".to_string(), Value::String(filter_value)),
        ("exit_status".to_string(), Value::Number(exit_status.into())),
        ("plugin_output".to_string(), Value::String(plugin_output)),
        (
            "performance_data".to_string(),
            Value::Array(perf_data.into_iter().map(Value::String).collect()),
        ),
        (
            "check_source".to_string(),
            Value::String(check_source.to_string()),
        ),
    ])
}

pub fn send_passive_check(
    check_source: &str,
    check_name: &str,
    check_host: &str,
    check_type: &str,
    check_data: &CheckResult,
    icinga_config: &IcingaConfig,
) {
    let client = Client::new();
    let data = format_check_payload(check_source, check_type, check_name, check_data);

    let response = client
        .post(&icinga_config.api_url)
        .basic_auth(&icinga_config.api_user, Some(&icinga_config.api_password))
        .header(ACCEPT, "application/json")
        .json(&data)
        .send()
        .unwrap();

    let status = response.status();
    if status == StatusCode::OK {
        println!(
            "Successfully sent passive check result for host {} check {} host {}",
            check_source, check_name, check_host
        );
    } else {
        let error_body = response.text().unwrap();
        println!(
            "Failed to send passive check result for host {} check {}",
            check_source, check_host
        );
        println!("Status: {}", status);
        println!("Response body: {}", error_body);
        println!(
            "Request data was: {}",
            serde_json::to_string_pretty(&data).unwrap()
        );
    }
}

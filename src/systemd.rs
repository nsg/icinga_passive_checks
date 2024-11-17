use std::collections::HashMap;

pub fn generate_unit_content(description: &str, exec_start: &str) -> String {
    let mut sections: HashMap<&str, Vec<(&str, String)>> = HashMap::new();
    
    sections.insert("Unit", vec![
        ("Description", description.to_string()),
        ("After", "network-online.target".to_string()),
        ("Requires", "network-online.target".to_string()),
    ]);

    sections.insert("Service", vec![
        ("ExecStart", format!("{} --daemon", exec_start)),
        ("DynamicUser", "true".to_string()),
        ("NoNewPrivileges", "true".to_string()),
        ("ProtectSystem", "strict".to_string()),
        ("ProtectHome", "true".to_string()),
        ("PrivateDevices", "true".to_string()),
        ("PrivateTmp", "true".to_string()),
        ("RestrictSUIDSGID", "true".to_string()),
        ("RestrictNamespaces", "true".to_string()),
        ("RuntimeDirectory", "icinga_passive_checks".to_string()),
    ]);

    sections.insert("Install", vec![
        ("WantedBy", "multi-user.target".to_string()),
    ]);

    let mut content = String::new();
    
    for (section, pairs) in &sections {
        content.push_str(&format!("[{}]\n", section));
        for (key, value) in pairs {
            content.push_str(&format!("{}={}\n", key, value));
        }
        content.push('\n');
    }

    content.trim_end().to_string()
}

use serde::Serialize;

pub struct SystemdService {
    pub name: String,
    pub description: String,
    pub exec_start: String,
    pub after: Vec<String>,
}

#[derive(Serialize)]
struct UnitFile {
    #[serde(rename = "Unit")]
    unit: UnitSection,
    #[serde(rename = "Service")]
    service: ServiceSection,
    #[serde(rename = "Install")]
    install: InstallSection,
}

#[derive(Serialize)]
struct UnitSection {
    #[serde(rename = "Description")]
    description: String,
    #[serde(rename = "After")]
    after: String,
}

#[derive(Serialize)]
struct ServiceSection {
    #[serde(rename = "ExecStart")]
    exec_start: String,
    #[serde(rename = "DynamicUser")]
    dynamic_user: bool,
    #[serde(rename = "NoNewPrivileges")]
    no_new_privileges: bool,
    #[serde(rename = "ProtectSystem")]
    protect_system: String,
    #[serde(rename = "ProtectHome")]
    protect_home: bool,
    #[serde(rename = "PrivateDevices")]
    private_devices: bool,
    #[serde(rename = "PrivateTmp")]
    private_tmp: bool,
    #[serde(rename = "RestrictSUIDSGID")]
    restrict_suid_sgid: bool,
}

#[derive(Serialize)]
struct InstallSection {
    #[serde(rename = "WantedBy")]
    wanted_by: String,
}

pub fn generate_unit_content(service: &SystemdService) -> String {
    let unit_file = UnitFile {
        unit: UnitSection {
            description: service.description.clone(),
            after: service.after.join(" "),
        },
        service: ServiceSection {
            exec_start: service.exec_start.clone(),
            dynamic_user: true,
            no_new_privileges: true,
            protect_system: "strict".to_string(),
            protect_home: true,
            private_devices: true,
            private_tmp: true,
            restrict_suid_sgid: true,
        },
        install: InstallSection {
            wanted_by: "multi-user.target".to_string(),
        },
    };

    toml::to_string(&unit_file).unwrap_or_default()
}

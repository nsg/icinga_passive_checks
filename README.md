# Icinga Passive Checks

A Rust-based utility for executing and submitting passive checks to Icinga2. The tool currently supports ping checks and is designed to be easily extensible for additional check types.

## Configuration (at server1)

Create a `config.toml` file with the following content:

```toml
[icinga]
api_url = "https://your-icinga-server/v1/actions/process-check-result"
api_user = "your-api-user"
api_password = "your-api-password"

[command]
debug = false

[daemon]
sleep_duration = 300

[[ping]]
name = "router"
host = "192.168.1.1"

[[ping]]
name = "webserver"
host = "10.0.0.5"
```

## Icinga configuration

I use something like below. The important part is that the host need to match the hostname, and the services need to match `Passive Ping: {name}` for ping checks.

```
object Host "server1" {
  import "passive-host"

  vars.passive_ping_host = [
    "router",
    "webserver"
  ]
}

apply Service "Passive Ping: " for (config in host.vars.passive_ping_host) {
  import "generic-service"

  enable_active_checks = true
  enable_passive_checks = true
  check_command = "dummy"
  check_interval = 10m

  vars.dummy_text = "No Passive Check Result Received"
  vars.dummy_state = "3"
}
```

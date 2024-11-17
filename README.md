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

## Report from scripts

```bash
icinga_passive_checks --control --check zpool --status 0 --message 'Pool OK'
```

If you prefer to use NC instead to talk with the control socket directly do this:

```bash
echo "report|$HOSTNAME|zpool|0|Pool OK" | nc -U /run/icinga_passive_checks/control.sock
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

  vars.passive_command_host = [
    "zpool",
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

apply Service "Passive Command: " for (config in host.vars.passive_command_host) {
  import "generic-service"

  enable_active_checks = true
  enable_passive_checks = true
  check_command = "dummy"
  check_interval = 10m

  vars.dummy_text = "No Passive Check Result Received"
  vars.dummy_state = "3"
}
```

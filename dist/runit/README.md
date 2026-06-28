# Running telemetroid under runit (Void Linux)

Three runit services:

| Service                | Role                                                        |
|------------------------|-------------------------------------------------------------|
| `telemetroid`          | The bot itself (supervised, runs as root).                  |
| `telemetroid-weekly`   | Sends `SIGUSR1` (short SMART test) every Monday 03:00.       |
| `telemetroid-monthly`  | Sends `SIGUSR2` (scrub + long test) on the 1st at 04:00.     |

The scheduler services trigger the bot with `sv 1` / `sv 2`, which deliver
`SIGUSR1` / `SIGUSR2` to the supervised process — no PID file needed.

## Prerequisites

```sh
xbps-install -S snooze        # cron-style scheduler used by the timer services
cargo build --release         # produces target/release/telemetroid
install -m 755 target/release/telemetroid /usr/local/bin/telemetroid
```

`smartmontools` (smartctl) and `btrfs-progs` must also be installed.

## Install

```sh
# Copy the service definitions
cp -r dist/runit/telemetroid           /etc/sv/
cp -r dist/runit/telemetroid-weekly    /etc/sv/
cp -r dist/runit/telemetroid-monthly   /etc/sv/

# Make the run scripts executable
chmod +x /etc/sv/telemetroid/run /etc/sv/telemetroid/log/run \
         /etc/sv/telemetroid-weekly/run /etc/sv/telemetroid-monthly/run

# Logs (svlogd target)
mkdir -p /var/log/telemetroid

# Configuration (NOT committed — create on the host)
mkdir -p /etc/sv/telemetroid/env
printf '%s' '<bot-token>' > /etc/sv/telemetroid/env/TELOXIDE_TOKEN
printf '%s' '<chat-id>'   > /etc/sv/telemetroid/env/TELEMETROID_ADMIN_CHAT
chmod 600 /etc/sv/telemetroid/env/TELOXIDE_TOKEN

# Enable (runsvdir watches /var/service)
ln -s /etc/sv/telemetroid          /var/service/
ln -s /etc/sv/telemetroid-weekly   /var/service/
ln -s /etc/sv/telemetroid-monthly  /var/service/
```

`TELEMETROID_ADMIN_CHAT` is the Telegram chat that scheduled-run reports are sent
to. Without it the bot still serves commands, but the timer services have nowhere
to report.

## Operating

```sh
sv status telemetroid            # check the bot
sv 1 telemetroid                 # trigger a short test now (same as the weekly timer)
sv 2 telemetroid                 # trigger scrub + long test now (same as the monthly timer)
sv restart telemetroid           # restart after deploying a new binary
tail -F /var/log/telemetroid/current
```

## Changing the schedule

Edit the `snooze` flags in each timer's `run` script:
`-w` day-of-week (0/7=Sun, 1=Mon …), `-d` day-of-month, `-H` hour, `-M` minute.
Unspecified smaller time fields default to 0, so the job fires once at that exact
minute. After editing, `sv restart telemetroid-weekly` (or `-monthly`).

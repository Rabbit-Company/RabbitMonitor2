# RabbitMonitor2

Rabbit Monitor is a simple program written in Rust that collects computer stats every 5 seconds (By default) and create [/metrics](https://openmetrics.io/) API endpoint for other programs like [Prometheus](https://prometheus.io/), [Grafana](https://grafana.com/)... to collect and display them.

API Endpoints:
- [/metrics](https://openmetrics.io/)

# Installation

```bash
# Download the binary
wget https://github.com/Rabbit-Company/RabbitMonitor2/releases/download/v3.1.0/rabbitmonitor
# Set file permissions
sudo chmod 777 rabbitmonitor
# Place the binary to `/usr/local/bin`
sudo mv rabbitmonitor /usr/local/bin
# Start the monitor
rabbitmonitor
```

# Daemonizing (using systemd)

Running Rabbit Monitor in the background is a simple task, just make sure that it runs without errors before doing this. Place the contents below in a file called `rabbitmonitor.service` in the `/etc/systemd/system/` directory.

```service
[Unit]
Description=Rabbit Monitor 
After=network.target

[Service]
Type=simple
User=root
ExecStart=rabbitmonitor
TimeoutStartSec=0
TimeoutStopSec=2
RemainAfterExit=yes

[Install]
WantedBy=multi-user.target
```
Then, run the commands below to reload systemd and start Rabbit Monitor.
```yml
systemctl enable --now rabbitmonitor
```

# Upgrade
```bash
# Stop service
systemctl stop rabbitmonitor

# Download Rabbit Monitor
wget https://github.com/Rabbit-Company/RabbitMonitor2/releases/download/v3.1.0/rabbitmonitor
sudo chmod 777 rabbitmonitor
sudo mv rabbitmonitor /usr/local/bin

# Start service
systemctl start rabbitmonitor
```

# Grafana Dashboard
Rabbit Monitor has a pre-made Grafana dashboard that looks like this:

![Grafana Dashboard](https://raw.githubusercontent.com/Rabbit-Company/RabbitMonitor2/main/images/1.png)

It can be installed from official Grafana website: [https://grafana.com/grafana/dashboards/18562](https://grafana.com/grafana/dashboards/18562)

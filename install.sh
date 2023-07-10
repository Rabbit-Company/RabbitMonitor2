#!/bin/bash

#note port is set to 8088 

# Download the binary
wget https://github.com/Rabbit-Company/RabbitMonitor2/releases/download/v4.1.0/rabbitmonitor

# Set file permissions
chmod 777 rabbitmonitor

# Place the binary to `/usr/local/bin`
sudo mv rabbitmonitor /usr/local/bin

# Create systemd service file
echo "[Unit]
Description=Rabbit Monitor 
After=network.target

[Service]
Type=simple
User=root
ExecStart=rabbitmonitor --port $port
TimeoutStartSec=0
TimeoutStopSec=2
RemainAfterExit=yes

[Install]
WantedBy=multi-user.target" | sudo tee /etc/systemd/system/rabbitmonitor.service

# Reload systemd
sudo systemctl daemon-reload

# Enable and start Rabbit Monitor
sudo systemctl enable --now rabbitmonitor
sudo systemctl start rabbitmonitor

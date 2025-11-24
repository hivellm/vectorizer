#!/bin/bash
# Vectorizer GUI Linux Post-Install Script

set -e

APP_NAME="vectorizer-gui"
INSTALL_DIR="/opt/vectorizer"
BIN_DIR="/usr/local/bin"
SERVICE_FILE="/etc/systemd/system/vectorizer.service"
DESKTOP_FILE="/usr/share/applications/vectorizer-gui.desktop"

echo "Installing Vectorizer GUI for Linux..."

# Create directories
mkdir -p "$INSTALL_DIR/config"
mkdir -p "$INSTALL_DIR/data"
mkdir -p "$INSTALL_DIR/backups"

# Copy vectorizer binary
if [ -f "./vectorizer" ]; then
    cp ./vectorizer "$INSTALL_DIR/vectorizer"
    chmod +x "$INSTALL_DIR/vectorizer"
fi

# Create symlink
ln -sf "$INSTALL_DIR/$APP_NAME" "$BIN_DIR/$APP_NAME"

# Create default config if not exists
if [ ! -f "$INSTALL_DIR/config/config.yml" ]; then
    cat > "$INSTALL_DIR/config/config.yml" << 'EOF'
server:
  host: "0.0.0.0"
  port: 15002

storage:
  data_dir: "/opt/vectorizer/data"
  cache_size: 1024

embedding:
  provider: "fastembed"
  model: "BAAI/bge-small-en-v1.5"
  dimension: 384

performance:
  threads: 4
  batch_size: 100
EOF
fi

# Create systemd service
cat > "$SERVICE_FILE" << EOF
[Unit]
Description=Vectorizer Vector Database
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=$INSTALL_DIR
ExecStart=$INSTALL_DIR/vectorizer
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
EOF

# Create desktop entry
cat > "$DESKTOP_FILE" << EOF
[Desktop Entry]
Name=Vectorizer GUI
Comment=Vector Database Management Tool
Exec=$BIN_DIR/$APP_NAME
Icon=$INSTALL_DIR/icon.png
Terminal=false
Type=Application
Categories=Development;Database;
EOF

# Reload systemd and enable service
systemctl daemon-reload
systemctl enable vectorizer.service

echo "✅ Installation complete!"
echo ""
echo "To start Vectorizer service:"
echo "  sudo systemctl start vectorizer"
echo ""
echo "To start Vectorizer GUI:"
echo "  $APP_NAME"
echo ""
echo "Service will start automatically on boot."


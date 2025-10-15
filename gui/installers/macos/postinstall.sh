#!/bin/bash
# Vectorizer GUI macOS Post-Install Script

set -e

APP_NAME="Vectorizer GUI"
INSTALL_DIR="/Applications/Vectorizer GUI.app"
SUPPORT_DIR="$HOME/Library/Application Support/Vectorizer"
LAUNCH_AGENT="$HOME/Library/LaunchAgents/io.hivellm.vectorizer.plist"

echo "Installing Vectorizer for macOS..."

# Create support directories
mkdir -p "$SUPPORT_DIR/config"
mkdir -p "$SUPPORT_DIR/data"
mkdir -p "$SUPPORT_DIR/backups"

# Copy vectorizer binary to app bundle
if [ -f "./vectorizer" ]; then
    cp ./vectorizer "$INSTALL_DIR/Contents/MacOS/vectorizer"
    chmod +x "$INSTALL_DIR/Contents/MacOS/vectorizer"
fi

# Create default config if not exists
if [ ! -f "$SUPPORT_DIR/config/config.yml" ]; then
    cat > "$SUPPORT_DIR/config/config.yml" << 'EOF'
server:
  host: "0.0.0.0"
  port: 15002

storage:
  data_dir: "$HOME/Library/Application Support/Vectorizer/data"
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

# Create LaunchAgent plist for auto-start
cat > "$LAUNCH_AGENT" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>io.hivellm.vectorizer</string>
    
    <key>ProgramArguments</key>
    <array>
        <string>$INSTALL_DIR/Contents/MacOS/vectorizer</string>
    </array>
    
    <key>WorkingDirectory</key>
    <string>$SUPPORT_DIR</string>
    
    <key>RunAtLoad</key>
    <true/>
    
    <key>KeepAlive</key>
    <true/>
    
    <key>StandardOutPath</key>
    <string>$SUPPORT_DIR/vectorizer.log</string>
    
    <key>StandardErrorPath</key>
    <string>$SUPPORT_DIR/vectorizer.error.log</string>
</dict>
</plist>
EOF

# Set permissions
chmod 644 "$LAUNCH_AGENT"

echo "✅ Installation complete!"
echo ""
echo "To start Vectorizer daemon:"
echo "  launchctl load $LAUNCH_AGENT"
echo ""
echo "To start Vectorizer GUI:"
echo "  Open 'Vectorizer GUI' from Applications"
echo ""
echo "Vectorizer will start automatically on login."


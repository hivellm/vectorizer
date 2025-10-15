# Installation Guide - Vectorizer GUI

## Windows

### Prerequisites
- Windows 10 or later
- Administrator privileges (for service installation)

### Installation Steps

1. Download `Vectorizer-GUI-Setup-{version}.msi`

2. Run the installer (double-click the MSI file)

3. Follow the installation wizard:
   - Accept the license agreement
   - Choose installation directory (default: `C:\Program Files\Vectorizer GUI`)
   - Select components (GUI + Vectorizer Service)
   - Choose desktop/start menu shortcuts

4. The installer will:
   - Install Vectorizer GUI application
   - Install Vectorizer binary
   - Create Windows Service for Vectorizer
   - Add desktop shortcut
   - Add start menu entries

5. Launch Vectorizer GUI from desktop shortcut

### Service Management

The vectorizer runs as a Windows Service:

```powershell
# Start service
Start-Service VectorizerService

# Stop service
Stop-Service VectorizerService

# Restart service
Restart-Service VectorizerService

# Check status
Get-Service VectorizerService
```

### Manual Service Control

- Open Services (Win + R, type `services.msc`)
- Find "Vectorizer Vector Database"
- Right-click → Start/Stop/Restart

### Uninstall

1. Control Panel → Programs and Features
2. Find "Vectorizer GUI"
3. Click Uninstall
4. Choose whether to keep data and configuration

---

## macOS

### Prerequisites
- macOS 10.14 (Mojave) or later
- Administrator privileges

### Installation Steps

1. Download `Vectorizer-GUI-{version}.dmg`

2. Open the DMG file

3. Drag "Vectorizer GUI" to Applications folder

4. First launch:
   - Right-click the app → Open (bypass Gatekeeper)
   - Or: System Preferences → Security & Privacy → Allow

5. The app will:
   - Create support directory: `~/Library/Application Support/Vectorizer`
   - Install LaunchAgent for Vectorizer daemon
   - Create default configuration

### Daemon Management

The vectorizer runs as a LaunchAgent:

```bash
# Start daemon
launchctl load ~/Library/LaunchAgents/io.hivellm.vectorizer.plist

# Stop daemon
launchctl unload ~/Library/LaunchAgents/io.hivellm.vectorizer.plist

# Check status
launchctl list | grep vectorizer
```

### Logs Location

- GUI logs: Console.app → Filter "Vectorizer"
- Vectorizer logs: `~/Library/Application Support/Vectorizer/vectorizer.log`

### Uninstall

```bash
# Stop daemon
launchctl unload ~/Library/LaunchAgents/io.hivellm.vectorizer.plist

# Remove app
rm -rf "/Applications/Vectorizer GUI.app"

# Remove daemon
rm ~/Library/LaunchAgents/io.hivellm.vectorizer.plist

# Remove data (optional)
rm -rf "~/Library/Application Support/Vectorizer"
```

---

## Linux

### Prerequisites
- Ubuntu 20.04+ / Debian 10+ / Fedora 30+ / Other modern Linux
- systemd (most distributions)
- sudo privileges

### Installation Steps (Debian/Ubuntu)

1. Download `vectorizer-gui_{version}_amd64.deb`

2. Install:
```bash
sudo dpkg -i vectorizer-gui_{version}_amd64.deb
sudo apt-get install -f  # Fix dependencies if needed
```

3. The installer will:
   - Install to `/opt/vectorizer`
   - Create systemd service
   - Add desktop entry
   - Create symlinks in `/usr/local/bin`

4. Launch from applications menu or run:
```bash
vectorizer-gui
```

### Service Management

The vectorizer runs as a systemd service:

```bash
# Start service
sudo systemctl start vectorizer

# Stop service
sudo systemctl stop vectorizer

# Restart service
sudo systemctl restart vectorizer

# Check status
sudo systemctl status vectorizer

# Enable auto-start
sudo systemctl enable vectorizer

# Disable auto-start
sudo systemctl disable vectorizer

# View logs
sudo journalctl -u vectorizer -f
```

### Data Locations

- Application: `/opt/vectorizer`
- Data: `/opt/vectorizer/data`
- Config: `/opt/vectorizer/config/config.yml`
- Backups: `/opt/vectorizer/backups`
- Logs: `journalctl -u vectorizer`

### Uninstall

```bash
# Stop and disable service
sudo systemctl stop vectorizer
sudo systemctl disable vectorizer

# Remove package
sudo apt-get remove vectorizer-gui

# Remove data (optional)
sudo rm -rf /opt/vectorizer
```

---

## Post-Installation

### First Run

1. Launch Vectorizer GUI

2. The GUI will check if Vectorizer is running:
   - If offline: Click "Start Vectorizer" button
   - If online: You're ready to go!

3. Default connection:
   - Host: `localhost`
   - Port: `15002`
   - No authentication

### Add Remote Connection

1. Click gear icon next to connection dropdown
2. Click "+ New Connection"
3. Fill in details:
   - Name: Your connection name
   - Type: Remote
   - Host: Remote server IP/hostname
   - Port: 15002 (or custom)
   - Token: Optional API token
4. Click "Test Connection"
5. Click "Save"
6. Select the connection from dropdown

### Create First Collection

1. Navigate to Collections (sidebar)
2. Click "+ Create Collection"
3. Fill in:
   - Name: my-docs
   - Dimension: 384
   - Metric: cosine
4. Click "Create"

### Index First Directory

1. Navigate to Workspace
2. Click "+ Add Directory"
3. Browse to select directory
4. Choose target collection
5. Enable "Auto-index file changes"
6. Click "Add Directory"

The GUI will automatically index all files and keep them synchronized!

---

## Troubleshooting

### Vectorizer Won't Start

**Windows:**
- Check Service status in Services panel
- Check Event Viewer for errors
- Verify port 15002 is not in use: `netstat -an | findstr 15002`

**macOS:**
- Check LaunchAgent: `launchctl list | grep vectorizer`
- View logs: `tail -f ~/Library/Application\ Support/Vectorizer/vectorizer.log`

**Linux:**
- Check service: `sudo systemctl status vectorizer`
- View logs: `sudo journalctl -u vectorizer -f`

### Connection Failed

1. Verify Vectorizer is running
2. Check firewall settings
3. Verify port (default: 15002)
4. For remote: ensure server is accessible

### Permission Errors

**Windows:**
- Run as Administrator

**macOS/Linux:**
- Check file permissions in data directory
- Ensure user has read/write access

---

## Configuration

Edit configuration from GUI:
1. Navigate to Configuration
2. Edit settings in tabs
3. Click "Save & Restart"

Or manually edit: `config.yml`

---

## Support

- GitHub Issues: https://github.com/hivellm/vectorizer/issues
- Documentation: https://github.com/hivellm/vectorizer#readme


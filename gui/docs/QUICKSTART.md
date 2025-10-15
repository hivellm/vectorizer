# Quick Start Guide - Vectorizer GUI

Get up and running with Vectorizer GUI in 5 minutes!

## Installation

### Windows
1. Download `Vectorizer-GUI-Setup-{version}.msi`
2. Run installer
3. Launch from desktop shortcut

### macOS
1. Download `Vectorizer-GUI-{version}.dmg`
2. Drag to Applications
3. Launch from Applications

### Linux
```bash
sudo dpkg -i vectorizer-gui_{version}_amd64.deb
vectorizer-gui
```

## First Run

### 1. Check Vectorizer Status

When you first open the GUI, you'll see the Dashboard.

**If Vectorizer is offline:**
- Click the **"Start Vectorizer"** button
- Wait ~5 seconds for it to start
- Status will change to "Online"

**If Vectorizer is online:**
- You're ready to go!

### 2. Create Your First Collection

1. Click **"New Collection"** in Quick Actions (or sidebar ➕ button)
2. Fill in details:
   - **Name**: `my-docs`
   - **Dimension**: `384` (default)
   - **Metric**: `cosine`
3. Click **"Create"**

### 3. Add a Directory to Index

1. Navigate to **Workspace** (left sidebar)
2. Click **"+ Add Directory"**
3. Click **"Browse"** and select a folder
4. Choose target collection: `my-docs`
5. Enable **"Auto-index file changes"** (recommended)
6. Click **"Add Directory"**

The GUI will automatically:
- Index all files in the directory
- Show progress in real-time
- Watch for file changes
- Auto-save periodically

### 4. Search Your Data

1. Click on **`my-docs`** in Collections sidebar
2. Enter a search query: `"machine learning concepts"`
3. Select search type: **Intelligent Search** (recommended)
4. Click **Search**

Results appear ranked by similarity!

## Common Tasks

### Connect to Remote Vectorizer

1. Click ⚙️ gear icon next to connection dropdown
2. Click **"+ New Connection"**
3. Fill in:
   - **Name**: `Production Server`
   - **Type**: Remote
   - **Host**: `192.168.1.100` (your server IP)
   - **Port**: `15002`
   - **Token**: (if authentication enabled)
4. Click **"Test Connection"**
5. If successful, click **"Save"**
6. Select the connection from dropdown

### Insert Text Manually

1. Navigate to a collection
2. Click **"Insert Data"**
3. Choose **"Text"**
4. Paste your text
5. Click **"Insert"**

### Create a Backup

1. Navigate to **Backups** (left sidebar)
2. Click **"+ Create Backup"**
3. Enter backup name
4. Select collections to backup
5. Click **"Create Backup"**

Backup saved in `./backups/` directory!

### Edit Configuration

1. Navigate to **Configuration** (left sidebar)
2. Edit settings in tabs:
   - **General**: Host, port, authentication
   - **Storage**: Data directory, cache size
   - **Embedding**: Provider, model, dimension
   - **Performance**: Threads, batch size
   - **YAML**: Raw configuration
3. Click **"Save & Restart"**

Vectorizer restarts with new config!

### View Logs

1. Navigate to **Logs** (left sidebar)
2. Use filters:
   - **Level**: Filter by log level (ERROR, WARN, INFO, DEBUG)
   - **Search**: Search log messages
   - **Lines**: Number of logs to show
3. Click **"Refresh"** to update
4. Click **"Export"** to save logs to file

## Keyboard Shortcuts

(To be implemented)

## Tips & Tricks

### Faster Searches
- Use **Intelligent Search** for best relevance
- Use **Semantic Search** for faster results
- Use **Basic Search** for exact matches

### Keep Data Safe
- Create backups before major changes
- Use **Force Save** if you manually modified files
- Enable auto-indexing for real-time updates

### Performance
- Smaller batch sizes = more responsive
- More threads = faster indexing (but more CPU)
- Larger cache = faster searches (but more memory)

### Remote Access
- Ensure firewall allows port 15002
- Use SSH tunnel for secure remote access:
  ```bash
  ssh -L 15002:localhost:15002 user@remote-server
  ```
- Connect to `localhost:15002` in GUI

## Troubleshooting

### Vectorizer Won't Start

**Check service status:**
- Windows: Services → VectorizerService
- Linux: `sudo systemctl status vectorizer`
- macOS: `launchctl list | grep vectorizer`

**Common fixes:**
- Port 15002 already in use → Change port in config
- Permission denied → Run as administrator/sudo
- Missing dependencies → Reinstall

### Connection Failed

1. Verify vectorizer is running
2. Check firewall settings
3. Ping the host
4. Verify port (default: 15002)
5. Check authentication token

### Indexing Slow

- Reduce `batch_size` in performance settings
- Check disk I/O
- Close other applications
- Increase `threads` if you have spare CPU

### Search No Results

- Check collection has vectors: View on Dashboard
- Try different search type
- Check query spelling
- Increase result limit

## Getting Help

- Documentation: See README.md
- Issues: GitHub Issues
- Logs: View in Logs page for detailed errors

## Next Steps

After getting started:
1. Index more directories
2. Create more collections for different data types
3. Set up remote connections
4. Create regular backups
5. Customize configuration for your needs

Enjoy using Vectorizer GUI! 🚀


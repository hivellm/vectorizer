# Security & Data Persistence

## Encrypted Storage

The Vectorizer GUI uses encrypted storage to protect sensitive data like connection credentials and API tokens.

### How It Works

1. **Encryption Key Generation**
   - On first run, a random 32-byte encryption key is generated
   - The key is stored in a separate secure location
   - All subsequent data is encrypted using this key

2. **Data Storage Location**
   - **Windows**: `%APPDATA%\vectorizer-config\config.json`
   - **macOS**: `~/Library/Application Support/vectorizer-config/config.json`
   - **Linux**: `~/.config/vectorizer-config/config.json`

3. **What's Encrypted**
   - Connection credentials (host, port, API tokens)
   - Workspace paths
   - User preferences
   - Active connection state

### Technology

- **electron-store**: Industry-standard encrypted storage for Electron apps
- **AES-256-GCM**: Strong encryption algorithm
- **Automatic encryption/decryption**: Transparent to the application

### Development Mode

When running in web development mode (`npm run dev:vite`), the app falls back to localStorage for convenience. **This data is NOT encrypted**. Always use Electron mode for production.

### Security Best Practices

1. **Keep your data directory secure**: The encryption key is only as secure as your OS user account
2. **Use strong OS passwords**: Full disk encryption is recommended
3. **API Tokens**: Tokens are encrypted at rest but transmitted over the network (use HTTPS in production)
4. **Backup considerations**: Encrypted config files cannot be restored without the encryption key

### Accessing Stored Data

Via the Electron API in renderer process:

```typescript
// Read encrypted data
const connections = await window.electron.getStoreValue('connections');

// Write encrypted data
await window.electron.setStoreValue('connections', connectionsArray);

// Delete encrypted data
await window.electron.deleteStoreValue('connections');

// Clear all data (requires confirmation)
await window.electron.clearStore();

// Get storage file path
const path = await window.electron.getStorePath();
```

### Resetting the Application

If you need to reset all data:

1. Close the application
2. Delete the config directory (see "Data Storage Location" above)
3. Restart the application - a new encryption key will be generated

## Data Migration

When migrating data between machines:

1. **Not recommended**: Copying the config file won't work as encryption keys are machine-specific
2. **Recommended approach**:
   - Export connections manually (feature to be implemented)
   - Import on the new machine
   - Re-enter sensitive credentials

## Compliance

- **GDPR**: User data is stored locally and encrypted
- **Data Portability**: Users have full access to their encrypted data files
- **Right to Erasure**: Users can delete all data by removing the config directory









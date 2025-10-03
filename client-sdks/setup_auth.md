# NPM Authentication Setup Guide - OTP Only

## Quick Setup (Recommended)

### Automated Authentication Script

Use our simplified authentication script that only requests OTP:

**Linux/Mac:**
```bash
./npm_auth_otp.sh
```

**Windows PowerShell:**
```powershell
.\npm_auth_otp.ps1
```

**Windows Command Prompt:**
```cmd
npm_auth_otp.bat
```

## Manual Setup Options

### Option 1: Interactive Login (Simplified)

The simplest way to authenticate is using npm's interactive login:

```bash
# Set browser for WSL environment
export BROWSER=wslview
npm login
```

You will be prompted for:
- Username
- Password
- Email
- **OTP (One-Time Password) - This is the main step**

### Option 2: Use NPM Token (Recommended for CI/CD)

1. **Create an NPM Token:**
   - Visit: https://www.npmjs.com/settings/tokens
   - Click "Generate New Token"
   - Choose "Automation" for automatic publishing
   - Copy the generated token

2. **Configure the Token:**
   ```bash
   # Windows (PowerShell)
   $env:NPM_TOKEN="your_token_here"
   
   # Linux/Mac
   export NPM_TOKEN="your_token_here"
   ```

3. **Create .npmrc file:**
   ```bash
   echo "//registry.npmjs.org/:_authToken=${NPM_TOKEN}" > .npmrc
   ```

### Option 3: Configure Browser for Authentication

1. **Set BROWSER variable:**
   ```bash
   # Windows
   set BROWSER=chrome
   # or
   set BROWSER=firefox
   
   # Linux/Mac
   export BROWSER=firefox
   # or
   export BROWSER=google-chrome
   ```

2. **Then try again:**
   ```bash
   npm publish
   ```

## Publishing Scripts

Our publishing scripts now include built-in OTP authentication:

```bash
# Publish all SDKs (will handle authentication automatically)
./publish_sdks.sh all

# Publish specific SDK
./publish_sdks.sh typescript
./publish_sdks.sh javascript
```

## Verification

After configuring, verify you're logged in:

```bash
npm whoami
```

## Configuration Files

- `.npmrc` - npm configuration
- `env.example` - Environment variables example
- `.env` - Your environment variables (not versioned)

## Troubleshooting

### Rollup Build Issues

If you encounter rollup build errors (like missing `@rollup/rollup-linux-x64-gnu`):

```bash
cd client-sdks/javascript
rm -rf node_modules package-lock.json
npm install
```

### Browser Issues

If npm tries to open a browser and fails:

```bash
export BROWSER=wslview
npm login
```

### Token Issues

If your token is invalid or expired:

1. Generate a new token from npmjs.com
2. Update your NPM_TOKEN environment variable
3. Update your ~/.npmrc file

### Permission Issues

Make sure your npm account has publish permissions for the package scope.

## Next Steps

1. Choose one of the authentication options above
2. Configure authentication using the automated script or manually
3. Run: `./publish_sdks.sh --test` to test
4. Run: `./publish_sdks.sh all` to publish all SDKs

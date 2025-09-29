# Authentication Setup for SDK Publishing

## Identified Problem
npm is trying to open a browser for authentication, but the `BROWSER` environment variable is not set.

## Available Solutions

### Option 1: Use NPM Token (Recommended for CI/CD)

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

### Option 2: Interactive Login

1. **Login to npm:**
   ```bash
   npm login
   ```

2. **Or use adduser:**
   ```bash
   npm adduser
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

## Automatic Configuration Script

You can use the `publish_sdks.sh` script with the `--setup-auth` option to configure automatically:

```bash
./publish_sdks.sh --setup-auth
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

## Next Steps

1. Choose one of the options above
2. Configure authentication
3. Run again: `./publish_sdks.sh --test` to test
4. Run: `./publish_sdks.sh --all` to publish

#!/bin/bash

# Script to configure authentication for SDK publishing

set -e

echo "🔐 Authentication Setup for SDK Publishing"
echo "=========================================="

# Check if already logged in
if npm whoami > /dev/null 2>&1; then
    echo "✅ Already logged in to npm as: $(npm whoami)"
    echo "🚀 Ready to publish!"
    exit 0
fi

echo "❌ Not logged in to npm"
echo ""
echo "Choose an option:"
echo "1) Interactive login (recommended for development)"
echo "2) Configure with NPM Token (recommended for CI/CD)"
echo "3) Configure browser for authentication"
echo "4) Exit"
echo ""

read -p "Enter your option (1-4): " option

case $option in
    1)
        echo "🔑 Performing interactive login to npm..."
        npm login
        if npm whoami > /dev/null 2>&1; then
            echo "✅ Login successful!"
            echo "🚀 Ready to publish!"
        else
            echo "❌ Login failed"
            exit 1
        fi
        ;;
    2)
        echo "🔑 Configuring with NPM Token..."
        echo ""
        echo "1. Visit: https://www.npmjs.com/settings/tokens"
        echo "2. Click 'Generate New Token'"
        echo "3. Choose 'Automation' for automatic publishing"
        echo "4. Copy the generated token"
        echo ""
        read -p "Paste your NPM Token here: " npm_token
        
        if [ -z "$npm_token" ]; then
            echo "❌ Token not provided"
            exit 1
        fi
        
        # Configure token
        echo "//registry.npmjs.org/:_authToken=${npm_token}" > .npmrc
        echo "✅ NPM Token configured!"
        
        # Verify
        if npm whoami > /dev/null 2>&1; then
            echo "✅ Authentication verified!"
            echo "🚀 Ready to publish!"
        else
            echo "❌ Invalid token"
            exit 1
        fi
        ;;
    3)
        echo "🌐 Configuring browser for authentication..."
        echo ""
        echo "Available browsers:"
        echo "1) firefox"
        echo "2) google-chrome"
        echo "3) chromium"
        echo "4) custom"
        echo ""
        read -p "Choose browser (1-4): " browser_option
        
        case $browser_option in
            1) BROWSER="firefox" ;;
            2) BROWSER="google-chrome" ;;
            3) BROWSER="chromium" ;;
            4) 
                read -p "Enter browser command: " custom_browser
                BROWSER="$custom_browser"
                ;;
            *) 
                echo "❌ Invalid option"
                exit 1
                ;;
        esac
        
        export BROWSER
        echo "export BROWSER=\"$BROWSER\"" >> ~/.bashrc
        echo "✅ Browser configured: $BROWSER"
        echo "🔄 Restart terminal or run: source ~/.bashrc"
        echo "🚀 Now you can try publishing again!"
        ;;
    4)
        echo "👋 Exiting..."
        exit 0
        ;;
    *)
        echo "❌ Invalid option"
        exit 1
        ;;
esac

echo ""
echo "📋 Next steps:"
echo "1. Run: ./publish_sdks.sh --test (to test)"
echo "2. Run: ./publish_sdks.sh --all (to publish all)"
echo "3. Or run: ./publish_sdks.sh --typescript (TypeScript only)"

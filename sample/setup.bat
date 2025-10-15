@echo off
echo 🚀 Setting up BitNet Chat Sample...

REM Check if Python is installed
python --version >nul 2>&1
if %errorlevel% neq 0 (
    echo ❌ Python is not installed. Please install Python 3.8 or higher.
    pause
    exit /b 1
)

REM Check if Node.js is installed
node --version >nul 2>&1
if %errorlevel% neq 0 (
    echo ❌ Node.js is not installed. Please install Node.js 16 or higher.
    pause
    exit /b 1
)

echo ✅ Python and Node.js found

REM Create virtual environment if it doesn't exist
if not exist "venv" (
    echo 📦 Creating Python virtual environment...
    python -m venv venv
    echo ✅ Virtual environment created
) else (
    echo ✅ Virtual environment already exists
)

REM Activate virtual environment and install dependencies
echo 📥 Installing Python dependencies...
call venv\Scripts\activate.bat
python -m pip install --upgrade pip
pip install -r requirements.txt

REM Install Node.js dependencies
echo 📥 Installing Node.js dependencies...
npm install

echo.
echo 🎉 Setup complete!
echo.
echo 📋 Next steps:
echo 1. Make sure Vectorizer is running: cargo run --bin vectorizer
echo 2. Start the chat server: npm start
echo 3. Open http://localhost:3000 in your browser
echo.
echo 📁 Model location: models\BitNet-b1.58-2B-4T\ggml-model-i2_s.gguf
echo    (Make sure the BitNet model file is in this location)
pause

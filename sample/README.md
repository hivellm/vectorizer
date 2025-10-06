# BitNet Chat with Vectorizer MCP

This is a simple chat application that demonstrates how to use BitNet (Microsoft's 1-bit language model) with Vectorizer as a Model Context Protocol (MCP) for knowledge retrieval.

## 🚀 Features

- **BitNet Integration**: Uses the BitNet b1.58 2B4T model for text generation
- **Vectorizer MCP**: Uses Vectorizer as a knowledge base through MCP
- **Real-time Chat**: Interactive web interface for chatting
- **Knowledge Search**: Automatically searches relevant knowledge for context
- **Modern UI**: Beautiful, responsive chat interface

## 📋 Prerequisites

1. **Node.js** (version 16 or higher)
2. **Rust** and **Cargo** (for Vectorizer)
3. **BitNet Model**: The model should be placed in `models/BitNet-b1.58-2B-4T/`

## 🛠️ Installation

### **Quick Setup (Recommended)**

**Linux/macOS:**
```bash
cd sample
./setup.sh
```

**Windows:**
```cmd
cd sample
setup.bat
```

### **Manual Setup**

1. **Create Python virtual environment:**
   ```bash
   cd sample
   python -m venv venv
   source venv/bin/activate  # Linux/macOS
   # OR
   venv\Scripts\activate     # Windows
   ```

2. **Install Python dependencies:**
   ```bash
   pip install -r requirements.txt
   ```

3. **Install Node.js dependencies:**
   ```bash
   npm install
   ```

4. **Ensure BitNet model is available:**
   The model file should be at:
   ```
   sample/models/BitNet-b1.58-2B-4T/ggml-model-i2_s.gguf
   ```

5. **Make sure Vectorizer is built and running:**
   ```bash
   cd ..
   cargo build --release
   cargo run --bin vectorizer
   ```

## 🚀 Usage

### **Prerequisites**
Make sure Vectorizer is running:
```bash
# In the main vectorizer directory
cargo run --bin vectorizer
```

### **Starting the Chat**

1. **Start the chat server:**
   ```bash
   cd sample
   npm start
   ```

2. **What happens on startup:**
   - ✅ Checks for BitNet model file
   - 🚀 Starts BitNet FastAPI server (Python with venv) on port 15003
   - 🔗 Connects to Vectorizer via MCP (SSE) on port 15002
   - 📁 Creates knowledge collection
   - 📚 Adds sample knowledge documents
   - 🌐 Starts Express.js web server on port 15004

3. **Open your browser:**
   Navigate to `http://localhost:15004`

4. **Start chatting:**
   - Ask questions about BitNet, Vectorizer, or MCP
   - The system will search the knowledge base for context
   - BitNet will generate responses based on the context

## 🔧 How It Works

### Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Web Browser   │    │   Express.js    │    │   Vectorizer    │
│                 │◄──►│   Chat Server   │◄──►│   (MCP Mode)    │
│   index.html    │    │   Port: 15004   │    │   Port: 15002   │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                                │
                                ▼
                       ┌─────────────────┐
                       │    BitNet       │
                       │   FastAPI       │
                       │   Port: 15003   │
                       └─────────────────┘
```

### Flow

1. **User sends message** → Web interface
2. **Server receives message** → Express.js endpoint
3. **Knowledge search** → Vectorizer MCP search
4. **Context retrieval** → Relevant documents found
5. **Response generation** → BitNet model (simulated)
6. **Response sent** → Back to user

## 📁 Project Structure

```
sample/
├── index.html              # Web chat interface
├── server.js               # Express.js server with BitNet integration
├── package.json            # Node.js dependencies
├── README.md               # This file
└── models/
    └── BitNet-b1.58-2B-4T/
        ├── ggml-model-i2_s.gguf    # BitNet model file
        └── README.md               # Model documentation
```

## 🎯 Sample Conversations

Try asking these questions:

- "What is BitNet?"
- "How does Vectorizer work?"
- "Tell me about Model Context Protocol"
- "What is quantization in machine learning?"

## 🔧 Configuration

### Vectorizer MCP Settings

You can modify these settings in `server.js`:

```javascript
const VECTORIZER_CONFIG = {
    host: 'localhost',
    port: 15002,
    collection: 'chat_knowledge'
};
```

### BitNet Model Path

The model path is configured in `server.js`:

```javascript
const MODEL_PATH = path.join(__dirname, 'models', 'BitNet-b1.58-2B-4T', 'ggml-model-i2_s.gguf');
```

## 🚨 Important Notes

### BitNet Implementation

**⚠️ Current Limitation**: This example uses a simplified response generation system rather than the actual BitNet model inference. In a production environment, you would need to:

1. **Install BitNet C++ library**: Follow the [official BitNet repository](https://github.com/microsoft/BitNet)
2. **Integrate with Node.js**: Use native bindings or spawn processes
3. **Handle model loading**: Load the GGUF model file properly

### Vectorizer Requirements

- Vectorizer server must be built and accessible
- The server will automatically start Vectorizer as a subprocess
- Make sure port 15002 is available

## 🔍 API Endpoints

### `GET /api/health`
Check server and Vectorizer status.

**Response:**
```json
{
  "status": "healthy",
  "vectorizer": true,
  "model": true
}
```

### `POST /api/chat`
Send a chat message.

**Request:**
```json
{
  "message": "What is BitNet?",
  "history": [
    {"role": "user", "content": "Hello"},
    {"role": "assistant", "content": "Hi there!"}
  ]
}
```

**Response:**
```json
{
  "response": "BitNet is a native 1-bit language model...",
  "searchResults": [...],
  "timestamp": "2025-01-05T12:00:00.000Z"
}
```

### `POST /api/search`
Search the knowledge base directly.

**Request:**
```json
{
  "query": "vector database"
}
```

## 🐛 Troubleshooting

### Common Issues

1. **"Model not found" error:**
   - Ensure the BitNet model file is in the correct location
   - Check the file path in `server.js`
   - Download the model from: [microsoft/bitnet-b1.58-2B-4T-gguf](https://huggingface.co/microsoft/bitnet-b1.58-2B-4T-gguf)

2. **"Virtual environment not found":**
   - Run the setup script: `./setup.sh` (Linux/macOS) or `setup.bat` (Windows)
   - Or manually create: `python -m venv venv`

3. **"Python not found" or "spawn python ENOENT":**
   - Make sure Python is installed and in PATH
   - The server now automatically activates the virtual environment
   - Try running: `python --version` to verify Python installation

4. **"Vectorizer connection failed":**
   - Make sure Vectorizer is running: `cargo run --bin vectorizer`
   - Check if port 15002 is available
   - Verify MCP endpoint: `http://localhost:15002/mcp/sse`

5. **"BitNet startup timeout":**
   - BitNet model loading takes time (up to 60 seconds)
   - Check Python dependencies: `pip install -r requirements.txt`
   - Ensure sufficient RAM (model needs ~2GB)

6. **"MCP connection failed":**
   - Verify Vectorizer is running with MCP support
   - Check SSE endpoint accessibility
   - Review Vectorizer logs for MCP startup messages

### Debug Mode

Run with debug logging:

```bash
DEBUG=* npm start
```

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Test thoroughly
5. Submit a pull request

## 📄 License

This project follows the same license as the main Vectorizer project.

## 🙏 Acknowledgments

- **Microsoft Research** for the BitNet model
- **Vectorizer Team** for the vector database
- **MCP Community** for the protocol specification

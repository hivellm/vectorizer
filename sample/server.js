const express = require('express');
const { spawn } = require('child_process');
const path = require('path');
const fs = require('fs');
const { Client } = require('@modelcontextprotocol/sdk/client/index.js');
const { SSEClientTransport } = require('@modelcontextprotocol/sdk/client/sse.js');

const app = express();
const PORT = 3000;

// Middleware
app.use(express.json());
app.use(express.static(path.join(__dirname)));

// BitNet model path
const MODEL_PATH = path.join(__dirname, 'models', 'BitNet-b1.58-2B-4T', 'ggml-model-i2_s.gguf');

// Global variables
let vectorizerProcess = null;
let bitnetProcess = null;
let isVectorizerReady = false;
let isBitNetReady = false;
let mcpClient = null;

// Vectorizer MCP configuration
const VECTORIZER_CONFIG = {
    host: 'localhost',
    port: 15002,
    collection: 'chat_knowledge'
};

// BitNet FastAPI configuration
const BITNET_CONFIG = {
    host: 'localhost',
    port: 8000
};

/**
 * Connect to Vectorizer via MCP (SSE)
 */
async function connectToVectorizerMCP() {
    console.log('🔗 Connecting to Vectorizer via MCP (SSE)...');
    
    try {
        // Connect to Vectorizer MCP server via SSE
        const transport = new SSEClientTransport({
            url: `http://${VECTORIZER_CONFIG.host}:${VECTORIZER_CONFIG.port}/mcp/sse`
        });

        mcpClient = new Client({
            name: 'bitnet-chat-client',
            version: '1.0.0'
        }, {
            capabilities: {
                resources: {},
                tools: {}
            }
        });

        await mcpClient.connect(transport);
        
        // Initialize the connection
        const initResult = await mcpClient.request({
            method: 'initialize',
            params: {
                protocolVersion: '2024-11-05',
                capabilities: {
                    resources: {},
                    tools: {}
                },
                clientInfo: {
                    name: 'bitnet-chat-client',
                    version: '1.0.0'
                }
            }
        }, {});

        console.log('✅ Connected to Vectorizer via MCP (SSE)!');
        console.log('📋 Server capabilities:', initResult.capabilities);
        
        isVectorizerReady = true;
        
    } catch (error) {
        console.error('❌ Failed to connect to Vectorizer MCP:', error);
        throw error;
    }
}

/**
 * Start BitNet FastAPI server
 */
async function startBitNetServer() {
    console.log('🚀 Starting BitNet FastAPI server...');
    
    return new Promise((resolve, reject) => {
        // Start BitNet FastAPI server
        bitnetProcess = spawn('python', ['bitnet_server.py'], {
            cwd: __dirname,
            stdio: ['pipe', 'pipe', 'pipe']
        });

        bitnetProcess.stdout.on('data', (data) => {
            const output = data.toString();
            console.log(`[BitNet] ${output}`);
            
            // Check if server is ready
            if (output.includes('Uvicorn running on') || output.includes('Application startup complete')) {
                isBitNetReady = true;
                console.log('✅ BitNet FastAPI server is ready!');
                resolve();
            }
        });

        bitnetProcess.stderr.on('data', (data) => {
            console.error(`[BitNet Error] ${data.toString()}`);
        });

        bitnetProcess.on('close', (code) => {
            console.log(`BitNet process exited with code ${code}`);
            isBitNetReady = false;
        });

        bitnetProcess.on('error', (error) => {
            console.error('Failed to start BitNet server:', error);
            reject(error);
        });

        // Timeout after 60 seconds (BitNet takes longer to load)
        setTimeout(() => {
            if (!isBitNetReady) {
                reject(new Error('BitNet startup timeout'));
            }
        }, 60000);
    });
}

/**
 * Check if BitNet model exists
 */
function checkBitNetModel() {
    if (!fs.existsSync(MODEL_PATH)) {
        throw new Error(`BitNet model not found at: ${MODEL_PATH}`);
    }
    console.log(`✅ BitNet model found: ${MODEL_PATH}`);
}

/**
 * Create collection in Vectorizer if it doesn't exist (via MCP)
 */
async function ensureCollectionExists() {
    try {
        if (!mcpClient) {
            throw new Error('MCP client not connected');
        }

        // List collections using MCP
        const listResult = await mcpClient.request({
            method: 'tools/call',
            params: {
                name: 'list_collections'
            }
        }, {});

        const collections = listResult.content?.[0]?.text ? JSON.parse(listResult.content[0].text) : [];
        
        if (!collections.find(col => col.name === VECTORIZER_CONFIG.collection)) {
            console.log(`📁 Creating collection: ${VECTORIZER_CONFIG.collection}`);
            
            // Create collection using MCP
            const createResult = await mcpClient.request({
                method: 'tools/call',
                params: {
                    name: 'create_collection',
                    arguments: {
                        name: VECTORIZER_CONFIG.collection,
                        dimension: 512,
                        metric: 'cosine'
                    }
                }
            }, {});

            console.log(`✅ Collection created: ${VECTORIZER_CONFIG.collection}`);
        } else {
            console.log(`✅ Collection already exists: ${VECTORIZER_CONFIG.collection}`);
        }
    } catch (error) {
        console.error('Error ensuring collection exists:', error);
        throw error;
    }
}

/**
 * Search in Vectorizer knowledge base (via MCP)
 */
async function searchKnowledgeBase(query) {
    try {
        if (!mcpClient) {
            throw new Error('MCP client not connected');
        }

        // Search using MCP
        const searchResult = await mcpClient.request({
            method: 'tools/call',
            params: {
                name: 'search_vectors',
                arguments: {
                    collection: VECTORIZER_CONFIG.collection,
                    query: query,
                    limit: 3
                }
            }
        }, {});

        // Parse the search results
        const results = searchResult.content?.[0]?.text ? JSON.parse(searchResult.content[0].text) : [];
        return results || [];
        
    } catch (error) {
        console.error('Knowledge base search error:', error);
        return [];
    }
}

/**
 * Generate response using BitNet FastAPI server
 */
async function generateBitNetResponse(messages, context = '') {
    try {
        // Prepare the request for BitNet
        const requestBody = {
            messages: messages,
            context: context,
            max_tokens: 512,
            temperature: 0.7,
            top_p: 0.9
        };

        const response = await fetch(`http://${BITNET_CONFIG.host}:${BITNET_CONFIG.port}/generate`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify(requestBody)
        });

        if (!response.ok) {
            throw new Error(`BitNet API error: ${response.status} ${response.statusText}`);
        }

        const data = await response.json();
        return data.response || data.text || 'Sorry, I could not generate a response.';

    } catch (error) {
        console.error('BitNet generation error:', error);
        
        // Fallback to simple responses if BitNet is not available
        const lastMessage = messages[messages.length - 1];
        const userQuery = lastMessage.content.toLowerCase();
        
        if (userQuery.includes('hello') || userQuery.includes('hi')) {
            return "Hello! I'm BitNet, but I'm having trouble connecting to my model right now. Please make sure the BitNet FastAPI server is running.";
        } else if (context) {
            return `Based on the knowledge I found: ${context.substring(0, 200)}... (Note: BitNet model is not responding)`;
        } else {
            return "I'm having trouble connecting to my BitNet model. Please check if the FastAPI server is running on port 8000.";
        }
    }
}

/**
 * Add sample knowledge to Vectorizer (via MCP)
 */
async function addSampleKnowledge() {
    const sampleDocuments = [
        {
            text: "BitNet is a native 1-bit Large Language Model developed by Microsoft Research. It uses 1.58-bit weights and achieves performance comparable to full-precision models while being much more efficient in terms of memory and computation.",
            metadata: { source: "bitnet_paper", topic: "machine_learning" }
        },
        {
            text: "Vectorizer is a high-performance vector database built in Rust. It supports multiple backends including CPU and GPU acceleration, and provides both REST API and MCP (Model Context Protocol) interfaces.",
            metadata: { source: "vectorizer_docs", topic: "vector_database" }
        },
        {
            text: "Model Context Protocol (MCP) is a standard for connecting AI models to external data sources and tools. It enables models to access real-time information and perform actions beyond their training data.",
            metadata: { source: "mcp_spec", topic: "ai_architecture" }
        },
        {
            text: "Quantization in machine learning reduces the precision of model weights and activations to decrease memory usage and improve inference speed. BitNet uses native quantization during training rather than post-training quantization.",
            metadata: { source: "quantization_guide", topic: "optimization" }
        }
    ];

    console.log('📚 Adding sample knowledge to Vectorizer via MCP...');
    
    for (const doc of sampleDocuments) {
        try {
            if (!mcpClient) {
                throw new Error('MCP client not connected');
            }

            // Insert document using MCP
            const insertResult = await mcpClient.request({
                method: 'tools/call',
                params: {
                    name: 'insert_text',
                    arguments: {
                        collection: VECTORIZER_CONFIG.collection,
                        text: doc.text,
                        metadata: doc.metadata
                    }
                }
            }, {});

            console.log(`✅ Added document: ${doc.metadata.source}`);
            
        } catch (error) {
            console.error('Error adding sample knowledge:', error);
        }
    }
    
    console.log('✅ Sample knowledge added to Vectorizer via MCP');
}

// Routes

/**
 * Health check endpoint
 */
app.get('/api/health', async (req, res) => {
    let bitnetStatus = false;
    let mcpStatus = false;
    
    // Check if BitNet server is responding
    try {
        const bitnetResponse = await fetch(`http://${BITNET_CONFIG.host}:${BITNET_CONFIG.port}/health`, {
            method: 'GET',
            timeout: 5000
        });
        bitnetStatus = bitnetResponse.ok;
    } catch (error) {
        bitnetStatus = false;
    }
    
    // Check if MCP client is connected
    try {
        if (mcpClient) {
            // Try to list collections to test MCP connection
            await mcpClient.request({
                method: 'tools/call',
                params: {
                    name: 'list_collections'
                }
            }, {});
            mcpStatus = true;
        }
    } catch (error) {
        mcpStatus = false;
    }
    
    res.json({ 
        status: 'healthy', 
        vectorizer: isVectorizerReady,
        mcp: mcpStatus,
        bitnet: bitnetStatus,
        model: fs.existsSync(MODEL_PATH)
    });
});

/**
 * Chat endpoint
 */
app.post('/api/chat', async (req, res) => {
    try {
        const { message, history } = req.body;
        
        if (!message) {
            return res.status(400).json({ error: 'Message is required' });
        }

        // Search knowledge base for relevant context
        const searchResults = await searchKnowledgeBase(message);
        
        // Generate context from search results
        const context = searchResults.length > 0 
            ? searchResults.map(result => result.content).join('\n')
            : '';

        // Generate response using BitNet
        const response = await generateBitNetResponse(history || [], context);

        res.json({
            response: response,
            searchResults: searchResults,
            timestamp: new Date().toISOString()
        });

    } catch (error) {
        console.error('Chat error:', error);
        res.status(500).json({ 
            error: 'Internal server error',
            message: error.message 
        });
    }
});

/**
 * Knowledge base search endpoint
 */
app.post('/api/search', async (req, res) => {
    try {
        const { query } = req.body;
        
        if (!query) {
            return res.status(400).json({ error: 'Query is required' });
        }

        const results = await searchKnowledgeBase(query);
        
        res.json({
            results: results,
            query: query,
            timestamp: new Date().toISOString()
        });

    } catch (error) {
        console.error('Search error:', error);
        res.status(500).json({ 
            error: 'Internal server error',
            message: error.message 
        });
    }
});

/**
 * Serve the main HTML file
 */
app.get('/', (req, res) => {
    res.sendFile(path.join(__dirname, 'index.html'));
});

/**
 * Initialize server
 */
async function initialize() {
    try {
        console.log('🤖 Starting BitNet Chat Server...');
        
        // Check BitNet model
        checkBitNetModel();
        
        // Start BitNet FastAPI server
        await startBitNetServer();
        
        // Connect to Vectorizer via MCP (assuming it's already running)
        await connectToVectorizerMCP();
        
        // Wait a bit for services to be fully ready
        await new Promise(resolve => setTimeout(resolve, 2000));
        
        // Ensure collection exists
        await ensureCollectionExists();
        
        // Add sample knowledge
        await addSampleKnowledge();
        
        // Start Express server
        app.listen(PORT, () => {
            console.log(`🌐 Chat server running on http://localhost:${PORT}`);
            console.log(`📁 Model path: ${MODEL_PATH}`);
            console.log(`🔗 Vectorizer MCP: http://localhost:${VECTORIZER_CONFIG.port}/mcp/sse`);
            console.log(`🤖 BitNet FastAPI: http://localhost:${BITNET_CONFIG.port}`);
            console.log(`📚 Collection: ${VECTORIZER_CONFIG.collection}`);
        });

    } catch (error) {
        console.error('❌ Initialization failed:', error);
        process.exit(1);
    }
}

/**
 * Graceful shutdown
 */
process.on('SIGINT', async () => {
    console.log('\n🛑 Shutting down gracefully...');
    
    if (bitnetProcess) {
        console.log('🛑 Stopping BitNet server...');
        bitnetProcess.kill();
    }
    
    if (mcpClient) {
        console.log('🛑 Closing MCP connection...');
        try {
            await mcpClient.close();
        } catch (error) {
            console.error('Error closing MCP connection:', error);
        }
    }
    
    process.exit(0);
});

process.on('SIGTERM', async () => {
    console.log('\n🛑 Shutting down gracefully...');
    
    if (bitnetProcess) {
        console.log('🛑 Stopping BitNet server...');
        bitnetProcess.kill();
    }
    
    if (mcpClient) {
        console.log('🛑 Closing MCP connection...');
        try {
            await mcpClient.close();
        } catch (error) {
            console.error('Error closing MCP connection:', error);
        }
    }
    
    process.exit(0);
});

// Start the server
initialize();

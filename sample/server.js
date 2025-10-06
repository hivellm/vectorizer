const express = require('express');
const { spawn } = require('child_process');
const path = require('path');
const fs = require('fs');

const app = express();
const PORT = 3000;

// Middleware
app.use(express.json());
app.use(express.static(path.join(__dirname)));

// BitNet model path
const MODEL_PATH = path.join(__dirname, 'models', 'BitNet-b1.58-2B-4T', 'ggml-model-i2_s.gguf');

// Global variables
let vectorizerProcess = null;
let isVectorizerReady = false;

// Vectorizer MCP configuration
const VECTORIZER_CONFIG = {
    host: 'localhost',
    port: 15002,
    collection: 'chat_knowledge'
};

/**
 * Start Vectorizer server as MCP
 */
async function startVectorizer() {
    console.log('🚀 Starting Vectorizer server...');
    
    return new Promise((resolve, reject) => {
        // Start Vectorizer server
        vectorizerProcess = spawn('cargo', ['run', '--bin', 'vectorizer'], {
            cwd: path.join(__dirname, '..'),
            stdio: ['pipe', 'pipe', 'pipe']
        });

        vectorizerProcess.stdout.on('data', (data) => {
            const output = data.toString();
            console.log(`[Vectorizer] ${output}`);
            
            // Check if server is ready
            if (output.includes('Server running on') || output.includes('Listening on')) {
                isVectorizerReady = true;
                console.log('✅ Vectorizer server is ready!');
                resolve();
            }
        });

        vectorizerProcess.stderr.on('data', (data) => {
            console.error(`[Vectorizer Error] ${data.toString()}`);
        });

        vectorizerProcess.on('close', (code) => {
            console.log(`Vectorizer process exited with code ${code}`);
            isVectorizerReady = false;
        });

        vectorizerProcess.on('error', (error) => {
            console.error('Failed to start Vectorizer:', error);
            reject(error);
        });

        // Timeout after 30 seconds
        setTimeout(() => {
            if (!isVectorizerReady) {
                reject(new Error('Vectorizer startup timeout'));
            }
        }, 30000);
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
 * Create collection in Vectorizer if it doesn't exist
 */
async function ensureCollectionExists() {
    try {
        // Check if collection exists
        const response = await fetch(`http://${VECTORIZER_CONFIG.host}:${VECTORIZER_CONFIG.port}/collections`);
        
        if (!response.ok) {
            throw new Error(`Failed to list collections: ${response.statusText}`);
        }

        const data = await response.json();
        const collections = data.collections || [];
        
        if (!collections.find(col => col.name === VECTORIZER_CONFIG.collection)) {
            console.log(`📁 Creating collection: ${VECTORIZER_CONFIG.collection}`);
            
            // Create collection
            const createResponse = await fetch(`http://${VECTORIZER_CONFIG.host}:${VECTORIZER_CONFIG.port}/collections`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({
                    name: VECTORIZER_CONFIG.collection,
                    dimension: 512,
                    metric: 'cosine'
                })
            });

            if (!createResponse.ok) {
                throw new Error(`Failed to create collection: ${createResponse.statusText}`);
            }

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
 * Search in Vectorizer knowledge base
 */
async function searchKnowledgeBase(query) {
    try {
        const response = await fetch(`http://${VECTORIZER_CONFIG.host}:${VECTORIZER_CONFIG.port}/collections/${VECTORIZER_CONFIG.collection}/search`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({
                query: query,
                limit: 3,
                filter: {}
            })
        });

        if (!response.ok) {
            throw new Error(`Search failed: ${response.statusText}`);
        }

        const data = await response.json();
        return data.results || [];
    } catch (error) {
        console.error('Knowledge base search error:', error);
        return [];
    }
}

/**
 * Generate response using BitNet (simplified implementation)
 * In a real implementation, you would use the actual BitNet C++ library
 */
async function generateBitNetResponse(messages, context = '') {
    // This is a simplified implementation
    // In reality, you would integrate with the actual BitNet model
    
    const lastMessage = messages[messages.length - 1];
    const userQuery = lastMessage.content.toLowerCase();
    
    // Simple response generation based on patterns
    let response = '';
    
    if (userQuery.includes('hello') || userQuery.includes('hi')) {
        response = "Hello! I'm BitNet, a 1-bit language model. How can I help you today?";
    } else if (userQuery.includes('what') && userQuery.includes('bitnet')) {
        response = "BitNet is a native 1-bit Large Language Model developed by Microsoft Research. It uses 1.58-bit weights and 8-bit activations, achieving performance comparable to full-precision models while being much more efficient.";
    } else if (userQuery.includes('help')) {
        response = "I can help you with various tasks! You can ask me questions, and I can also search through a knowledge base using Vectorizer. What would you like to know?";
    } else if (userQuery.includes('vectorizer')) {
        response = "Vectorizer is a high-performance vector database that I'm using as a Model Context Protocol (MCP) to search through knowledge and provide more accurate responses.";
    } else if (context) {
        response = `Based on the knowledge I found: ${context.substring(0, 200)}...`;
    } else {
        response = "That's an interesting question! While I don't have specific information about that topic in my knowledge base, I'd be happy to help you explore it further. Could you provide more details?";
    }
    
    // Simulate some processing time
    await new Promise(resolve => setTimeout(resolve, 1000 + Math.random() * 2000));
    
    return response;
}

/**
 * Add sample knowledge to Vectorizer
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

    console.log('📚 Adding sample knowledge to Vectorizer...');
    
    for (const doc of sampleDocuments) {
        try {
            const response = await fetch(`http://${VECTORIZER_CONFIG.host}:${VECTORIZER_CONFIG.port}/insert`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({
                    collection: VECTORIZER_CONFIG.collection,
                    text: doc.text,
                    metadata: doc.metadata
                })
            });

            if (!response.ok) {
                console.error(`Failed to add document: ${response.statusText}`);
            }
        } catch (error) {
            console.error('Error adding sample knowledge:', error);
        }
    }
    
    console.log('✅ Sample knowledge added to Vectorizer');
}

// Routes

/**
 * Health check endpoint
 */
app.get('/api/health', (req, res) => {
    res.json({ 
        status: 'healthy', 
        vectorizer: isVectorizerReady,
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
        
        // Start Vectorizer
        await startVectorizer();
        
        // Wait a bit for Vectorizer to be fully ready
        await new Promise(resolve => setTimeout(resolve, 2000));
        
        // Ensure collection exists
        await ensureCollectionExists();
        
        // Add sample knowledge
        await addSampleKnowledge();
        
        // Start Express server
        app.listen(PORT, () => {
            console.log(`🌐 Chat server running on http://localhost:${PORT}`);
            console.log(`📁 Model path: ${MODEL_PATH}`);
            console.log(`🔗 Vectorizer: http://localhost:${VECTORIZER_CONFIG.port}`);
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
process.on('SIGINT', () => {
    console.log('\n🛑 Shutting down gracefully...');
    
    if (vectorizerProcess) {
        vectorizerProcess.kill();
    }
    
    process.exit(0);
});

process.on('SIGTERM', () => {
    console.log('\n🛑 Shutting down gracefully...');
    
    if (vectorizerProcess) {
        vectorizerProcess.kill();
    }
    
    process.exit(0);
});

// Start the server
initialize();

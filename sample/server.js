const express = require('express');
const { spawn } = require('child_process');
const path = require('path');
const fs = require('fs');
const http = require('http');
const https = require('https');
const WebSocket = require('ws');

const app = express();
const server = http.createServer(app);
const wss = new WebSocket.Server({ server });
const PORT = 15004; // Keep this port for the Node.js server

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
let mcpChannel = null; // Our custom MCP-like channel
let mcpClient = null;

// Vectorizer MCP configuration
const VECTORIZER_CONFIG = {
    host: 'localhost',
    port: 15002,
    collection: 'chat_knowledge'
};

// BitNet FastAPI configuration (Nova versão)
const BITNET_CONFIG = {
    host: 'localhost',
    port: 15006
};

/**
 * Custom MCP-like Channel for Vectorizer communication
 */
class MCPChannel {
    constructor() {
        this.tools = {
            'list_collections': this.listCollections.bind(this),
            'search_vectors': this.searchVectors.bind(this),
            'get_collection_info': this.getCollectionInfo.bind(this),
            'insert_text': this.insertText.bind(this),
            'delete_vectors': this.deleteVectors.bind(this)
        };
    }

    async callTool(toolName, args) {
        if (!this.tools[toolName]) {
            throw new Error(`Unknown tool: ${toolName}`);
        }

        try {
            const result = await this.tools[toolName](args);
            return result;
        } catch (error) {
            throw error;
        }
    }

    async listCollections(args = {}) {
        const response = await makeHttpRequest('GET', '/collections');
        
        // Handle different response formats
        const collections = Array.isArray(response) ? response : 
                           response.collections || 
                           (response.data ? response.data.collections : []);
        
        return {
            content: [{
                type: 'text',
                text: JSON.stringify(collections)
            }]
        };
    }

    async searchVectors(args) {
        const { collection, query, limit = 3, min_score = 0.1 } = args; // Reduced limit to 3
        
        try {
            // Use the correct endpoint that actually works
            const response = await makeHttpRequest('GET', `/collections/${collection}/vectors`);

            // Handle different response formats
            let searchResults = [];
            if (Array.isArray(response)) {
                searchResults = response;
            } else if (response.vectors && Array.isArray(response.vectors)) {
                searchResults = response.vectors;
            } else if (response.results && Array.isArray(response.results)) {
                searchResults = response.results;
            } else if (response.data && Array.isArray(response.data)) {
                searchResults = response.data;
            } else {
                console.warn(`⚠️ Unexpected vectors response format for ${collection}`);
                return {
                    content: [{
                        type: 'text',
                        text: JSON.stringify([])
                    }]
                };
            }

            // Limit results to prevent memory issues
            const limitedResults = searchResults.slice(0, limit);

            // Format results for better context with content truncation
            const formattedResults = limitedResults.map(result => {
                let content = '';
                let metadata = {};
                
                if (typeof result === 'object' && result.payload && result.payload.content) {
                    content = result.payload.content;
                    metadata = result.payload.metadata || {};
                } else if (typeof result === 'object' && result.content) {
                    content = result.content;
                    metadata = result.metadata || {};
                } else {
                    content = JSON.stringify(result);
                }

                // Truncate content to prevent memory issues (max 1000 chars)
                if (content && content.length > 1000) {
                    content = content.substring(0, 1000) + '...';
                }

                return {
                    content: content,
                    score: result.score || 0.5,
                    metadata: metadata
                };
            });

            return {
                content: [{
                    type: 'text',
                    text: JSON.stringify(formattedResults)
                }]
            };

        } catch (error) {
            return {
                content: [{
                    type: 'text',
                    text: JSON.stringify([])
                }]
            };
        }
    }

    async getCollectionInfo(args) {
        const { collection } = args;
        
        const response = await makeHttpRequest('GET', `/collections/${collection}`);
        
        return {
            content: [{
                type: 'text',
                text: JSON.stringify(response)
            }]
        };
    }

    async insertText(args) {
        const { collection, text, metadata = {} } = args;
        
        const response = await makeHttpRequest('POST', `/collections/${collection}/insert`, {
            text: text,
            metadata: metadata
        });

        return {
            content: [{
                type: 'text',
                text: JSON.stringify(response)
            }]
        };
    }

    async deleteVectors(args) {
        const { collection, vector_ids } = args;
        
        const response = await makeHttpRequest('DELETE', `/collections/${collection}/vectors`, {
            vector_ids: vector_ids
        });

        return {
            content: [{
                type: 'text',
                text: JSON.stringify(response)
            }]
        };
    }
}

/**
 * Make HTTP request to Vectorizer
 */
function makeHttpRequest(method, path, data = null) {
    return new Promise((resolve, reject) => {
        const options = {
            hostname: VECTORIZER_CONFIG.host,
            port: VECTORIZER_CONFIG.port,
            path: path,
            method: method,
            headers: {
                'Content-Type': 'application/json',
                'Accept': 'application/json'
            }
        };

        if (data) {
            const jsonData = JSON.stringify(data);
            options.headers['Content-Length'] = Buffer.byteLength(jsonData);
        }

        const req = http.request(options, (res) => {
            let responseData = '';

            // Log response status
            console.log(`📡 HTTP ${method} ${path} -> ${res.statusCode} ${res.statusMessage}`);

            res.on('data', (chunk) => {
                responseData += chunk;
            });

            res.on('end', () => {
                console.log(`📄 Raw response data: "${responseData}"`);
                
                // Handle empty response
                if (!responseData || responseData.trim() === '') {
                    console.warn(`⚠️ Empty response for ${method} ${path}`);
                    resolve([]);
                    return;
                }

                try {
                    const jsonResponse = JSON.parse(responseData);
                    resolve(jsonResponse);
                } catch (error) {
                    console.error(`❌ JSON parse error for ${method} ${path}:`, error);
                    console.error(`Raw data: "${responseData}"`);
                    
                    // If it's not JSON, try to return as string or empty array
                    if (responseData.trim() === '""' || responseData.trim() === '') {
                        resolve([]);
                    } else {
                        resolve(responseData);
                    }
                }
            });
        });

        req.on('error', (error) => {
            console.error(`❌ HTTP request error for ${method} ${path}:`, error);
            reject(error);
        });

        // Set timeout
        req.setTimeout(10000, () => {
            req.destroy();
            reject(new Error('Request timeout'));
        });

        if (data) {
            req.write(JSON.stringify(data));
        }

        req.end();
    });
}

/**
 * Connect to Vectorizer via HTTP
 */
async function connectToVectorizerHTTP() {
    console.log('🔗 Connecting to Vectorizer via HTTP...');
    
    try {
        // Test connection to Vectorizer REST API
        const response = await makeHttpRequest('GET', '/health');
        
        if (response.status === 'healthy') {
            console.log('✅ Connected to Vectorizer HTTP successfully!');
            console.log('📋 Server status:', response);
            
            // Initialize MCP Channel
            mcpChannel = new MCPChannel();
            console.log('🔧 MCP Channel initialized with tools:', Object.keys(mcpChannel.tools));
            
            isVectorizerReady = true;
            return true;
        } else {
            throw new Error('Vectorizer health check failed');
        }
        
    } catch (error) {
        console.error('❌ Failed to connect to Vectorizer HTTP:', error.message);
        throw error;
    }
}

/**
 * Start BitNet FastAPI server
 */
async function startBitNetServer() {
    console.log('🚀 Starting BitNet FastAPI server...');
    
    return new Promise((resolve, reject) => {
        // Check if we're on Windows or Unix-like system
        const isWindows = process.platform === 'win32';
        
        let command, args;
        
        if (isWindows) {
            // Windows: use system Python directly
            command = 'python';
            args = ['bitnet_server.py'];
        } else {
            // Unix-like: use system Python directly
            command = 'python3';
            args = ['bitnet_server.py'];
        }
        
        console.log(`🐍 Using Python: ${command}`);
        
        // Start BitNet FastAPI server
        bitnetProcess = spawn(command, args, {
            cwd: __dirname,
            stdio: ['pipe', 'pipe', 'pipe']
        });

        bitnetProcess.stdout.on('data', (data) => {
            const output = data.toString();
            console.log(`[BitNet] ${output}`);
            
            // Check if server is ready - simplified detection
            if (output.includes('Uvicorn running on')) {
                isBitNetReady = true;
                console.log('✅ BitNet FastAPI server is ready!');
                resolve();
            }
        });

        bitnetProcess.stderr.on('data', (data) => {
            const output = data.toString();
            console.error(`[BitNet Error] ${output}`);
            
            // Check if server is ready - also check stderr for Uvicorn message
            if (output.includes('Uvicorn running on')) {
                isBitNetReady = true;
                console.log('✅ BitNet FastAPI server is ready!');
                resolve();
            }
        });

        bitnetProcess.on('close', (code) => {
            console.log(`BitNet process exited with code ${code}`);
            isBitNetReady = false;
        });

        bitnetProcess.on('error', (error) => {
            console.error('Failed to start BitNet server:', error);
            reject(error);
        });

        // Timeout after 90 seconds (BitNet takes longer to load)
        setTimeout(() => {
            if (!isBitNetReady) {
                reject(new Error('BitNet startup timeout'));
            }
        }, 90000);
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
 * Create collection in Vectorizer if it doesn't exist (via HTTP)
 */
async function ensureCollectionExists() {
    try {
        if (!isVectorizerReady) {
            throw new Error('Vectorizer not connected');
        }

        // List collections using HTTP
        const collectionsResponse = await makeHttpRequest('GET', '/collections');
        
        // Handle different response formats
        const collections = Array.isArray(collectionsResponse) ? collectionsResponse : 
                           collectionsResponse.collections || 
                           (collectionsResponse.data ? collectionsResponse.data.collections : []);
        
        if (!collections.find(col => col.name === VECTORIZER_CONFIG.collection)) {
            console.log(`📁 Creating collection: ${VECTORIZER_CONFIG.collection}`);
            
            // Create collection using HTTP
            await makeHttpRequest('POST', '/collections', {
                name: VECTORIZER_CONFIG.collection,
                dimension: 512,
                metric: 'cosine'
            });

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
 * Search in Vectorizer knowledge base (via MCP Channel)
 */
async function searchKnowledgeBase(query, collection = null, limit = 10) {
    try {
        if (!mcpChannel) {
            throw new Error('MCP Channel not initialized');
        }

        // Use multiple collections for broader context
        const collections = collection ? [collection] : [
            VECTORIZER_CONFIG.collection,
            'cmmv-core-docs',
            'cmmv-docs-en-content',
            'vectorizer-docs',
            'gov-bips'
        ];

        let allResults = [];

        // Search each collection
        for (const coll of collections) {
            try {
                console.log(`🔍 Searching collection: ${coll} for query: "${query}"`);

                const result = await mcpChannel.callTool('search_vectors', {
                    collection: coll,
                    query: query,
                    limit: 2, // Reduced to 2 per collection to minimize data
                    min_score: 0.1 // Higher threshold for better results
                });

                // Parse the result
                const results = JSON.parse(result.content[0].text);
                if (results && Array.isArray(results)) {
                    // Sanitize results to include only metadata and payload, no vectors
                    const sanitizedResults = results.map(r => {
                        // Extract only safe data for BitNet context
                        let content = r.content || '';
                        // Truncate content to reduce payload size
                        if (content.length > 300) {
                            content = content.substring(0, 300) + '...';
                        }

                        const sanitized = {
                            content: content,
                            score: r.score || 0.5,
                            metadata: r.metadata || {},
                            collection: coll,
                            source: coll
                        };

                        // Remove any vector data that might be present
                        if (r.vector) delete sanitized.vector;
                        if (r.vectors) delete sanitized.vectors;
                        if (r.embedding) delete sanitized.embedding;
                        if (r.embeddings) delete sanitized.embeddings;
                        if (r.payload && r.payload.vector) delete r.payload.vector;
                        if (r.payload && r.payload.vectors) delete r.payload.vectors;
                        if (r.payload && r.payload.embedding) delete r.payload.embedding;
                        if (r.payload && r.payload.embeddings) delete r.payload.embeddings;

                        // Also remove any other potential vector fields
                        const vectorFields = ['vector_data', 'embedding_data', 'vec', 'emb'];
                        vectorFields.forEach(field => {
                            if (r[field]) delete sanitized[field];
                            if (r.payload && r.payload[field]) delete r.payload[field];
                        });

                        // Remove any field that looks like a vector (array of numbers)
                        const isVectorArray = (arr) => Array.isArray(arr) && arr.length > 10 &&
                            arr.every(item => typeof item === 'number' && item >= -1 && item <= 1);

                        Object.keys(sanitized).forEach(key => {
                            if (isVectorArray(sanitized[key])) {
                                console.log(`Removing vector field: ${key}`);
                                delete sanitized[key];
                            }
                        });

                        if (sanitized.payload) {
                            Object.keys(sanitized.payload).forEach(key => {
                                if (isVectorArray(sanitized.payload[key])) {
                                    console.log(`Removing vector field from payload: ${key}`);
                                    delete sanitized.payload[key];
                                }
                            });
                        }

                        return sanitized;
                    });

                    allResults = allResults.concat(sanitizedResults);
                    console.log(`✅ Found ${results.length} results in ${coll}`);
                }
            } catch (collError) {
                console.warn(`⚠️ Collection ${coll} search failed:`, collError.message);
                // Continue with other collections
            }
        }

        // Sort by score and limit total results to prevent memory issues
        allResults.sort((a, b) => (b.score || 0) - (a.score || 0));
        const limitedResults = allResults.slice(0, Math.min(limit, 6)); // Max 6 total results

        console.log(`📊 Total search results: ${limitedResults.length} from ${collections.length} collections`);

        return limitedResults || [];

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
        // Ensure messages is an array and has at least one element
        if (!Array.isArray(messages) || messages.length === 0) {
            messages = [{ content: 'Hello' }];
        }
        
        // Get the last message safely
        const lastMessage = messages[messages.length - 1];
        const messageText = lastMessage?.content || lastMessage || 'Hello';

        const response = await fetch(`http://${BITNET_CONFIG.host}:${BITNET_CONFIG.port}/generate`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({
                messages: [{ role: "user", content: messageText }],
                context: context,
                max_tokens: 500
            }),
            signal: AbortSignal.timeout(120000) // 2 minutes timeout
        });

        if (!response.ok) {
            throw new Error(`BitNet API error: ${response.status} ${response.statusText}`);
        }

        const data = await response.json();
        return data.response || 'Sorry, I could not generate a response.';

    } catch (error) {
        console.error('BitNet generation error:', error);
        
        // Fallback to simple responses if BitNet is not available
        const lastMessage = messages[messages.length - 1];
        const userQuery = (lastMessage?.content || lastMessage || '').toLowerCase();
        
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
 * Add sample knowledge to Vectorizer (via HTTP)
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

    console.log('📚 Adding sample knowledge to Vectorizer via HTTP...');
    
    for (const doc of sampleDocuments) {
        try {
            if (!isVectorizerReady) {
                throw new Error('Vectorizer not connected');
            }

            // Insert document using HTTP
            await makeHttpRequest('POST', '/collections/' + VECTORIZER_CONFIG.collection + '/insert', {
                text: doc.text,
                metadata: doc.metadata
            });

            console.log(`✅ Added document: ${doc.metadata.source}`);
            
        } catch (error) {
            console.error('Error adding sample knowledge:', error);
        }
    }
    
    console.log('✅ Sample knowledge added to Vectorizer via HTTP');
}

// Routes

/**
 * Health check endpoint
 */
app.get('/api/health', async (req, res) => {
    let bitnetStatus = false;
    let vectorizerStatus = false;
    
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
    
    // Check if Vectorizer is responding
    try {
        // Try to get health status from Vectorizer directly
        const vectorizerResponse = await fetch(`http://${VECTORIZER_CONFIG.host}:${VECTORIZER_CONFIG.port}/health`, {
            method: 'GET',
            timeout: 5000
        });
        vectorizerStatus = vectorizerResponse.ok;
    } catch (error) {
        vectorizerStatus = false;
    }
    
    res.json({ 
        status: 'healthy', 
        vectorizer: vectorizerStatus,
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

        // Search knowledge base for relevant context (more collections, more results)
        const searchResults = await searchKnowledgeBase(message, null, 15);

        // Create rich context with technical details
        let context = '';
        if (searchResults.length > 0) {
            console.log(`📚 Building context from ${searchResults.length} search results`);

            const contextParts = searchResults.map((result, index) => {
                const collection = result.collection || 'unknown';
                const score = result.score ? result.score.toFixed(3) : '0.000';
                const content = result.content || '';

                // Include more technical details in context
                return `[${collection}] (score: ${score})\n${content}`;
            });

            context = contextParts.join('\n\n---\n\n');
            console.log(`📝 Context length: ${context.length} characters`);
        } else {
            console.warn('⚠️ No search results found for context');
        }

        // Generate response using BitNet
        const messages = history || [{ content: message }];
        const response = await generateBitNetResponse(messages, context);

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
 * WebSocket endpoint for streaming chat
 */
wss.on('connection', (ws) => {
    console.log('🔌 New WebSocket connection established');

    ws.on('message', async (data) => {
        try {
            const message = JSON.parse(data.toString());
            console.log('📨 WebSocket message received:', message.type);

            if (message.type === 'chat') {
                const { message: userMessage, history = [] } = message;

                // Send initial acknowledgment
                ws.send(JSON.stringify({
                    type: 'status',
                    status: 'searching',
                    message: 'Searching knowledge base...'
                }));

                try {
                    // Search knowledge base for relevant context
                    const searchResults = await searchKnowledgeBase(userMessage, null, 15);

                    ws.send(JSON.stringify({
                        type: 'status',
                        status: 'generating',
                        message: `Found ${searchResults.length} relevant results, generating response...`
                    }));

                    // Create rich context with technical details
                    let context = '';
                    if (searchResults.length > 0) {
                        console.log(`📚 Building context from ${searchResults.length} search results`);

                        const contextParts = searchResults.map((result, index) => {
                            const collection = result.collection || 'unknown';
                            const score = result.score ? result.score.toFixed(3) : '0.000';
                            const content = result.content || '';

                            // Include more technical details in context
                            return `[${collection}] (score: ${score})\n${content}`;
                        });

                        context = contextParts.join('\n\n---\n\n');
                        console.log(`📝 Context length: ${context.length} characters`);
                    }

                    // Stream the BitNet response (long-running operation)
                    const messages = history || [{ content: userMessage }];
                    await streamBitNetResponse(ws, messages, context);

                } catch (error) {
                    console.error('WebSocket chat error:', error);
                    ws.send(JSON.stringify({
                        type: 'error',
                        error: error.message
                    }));
                }
            }
        } catch (error) {
            console.error('WebSocket message error:', error);
            ws.send(JSON.stringify({
                type: 'error',
                error: 'Invalid message format'
            }));
        }
    });

    ws.on('close', () => {
        console.log('🔌 WebSocket connection closed');
    });

    ws.on('error', (error) => {
        console.error('WebSocket error:', error);
    });
});

/**
 * Stream BitNet response via WebSocket
 */
async function streamBitNetResponse(ws, messages, context = '') {
    try {
        console.log('🚀 Starting BitNet response streaming...');

        // Prepare the request for BitNet (nova versão usa formato diferente)

        // Use longer timeout for WebSocket streaming
        const controller = new AbortController();
        const timeoutId = setTimeout(() => controller.abort(), 600000); // 10 minutes

        const response = await fetch(`http://${BITNET_CONFIG.host}:${BITNET_CONFIG.port}/generate`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({
                messages: [{ role: "user", content: messages[messages.length - 1]?.content || messages[messages.length - 1] || '' }],
                context: context,
                max_tokens: 500
            }),
            signal: controller.signal,
            timeout: 300000 // 5 minutes for headers
        });

        clearTimeout(timeoutId);

        if (!response.ok) {
            throw new Error(`BitNet API error: ${response.status}`);
        }

        let responseData;

        try {
            responseData = await response.json();
        } catch (jsonError) {
            // If response is not valid JSON, treat it as plain text and send to chat for debugging
            console.warn('⚠️ BitNet returned non-JSON response, sending as debug to chat');
            const textResponse = await response.text();
            ws.send(JSON.stringify({
                type: 'response',
                response: `🔍 DEBUG: BitNet returned non-JSON response:\n\n${textResponse}`,
                timestamp: new Date().toISOString()
            }));
            console.log('✅ BitNet non-JSON response sent to chat for debugging');
            return;
        }

        // Send the complete response (nova versão retorna data.response)
        ws.send(JSON.stringify({
            type: 'response',
            response: responseData.response || responseData,
            timestamp: new Date().toISOString()
        }));

        console.log('✅ BitNet response streaming completed');

    } catch (error) {
        console.error('BitNet streaming error:', error);

        if (error.name === 'AbortError') {
            ws.send(JSON.stringify({
                type: 'error',
                error: 'Response generation timed out after 5 minutes'
            }));
        } else {
            ws.send(JSON.stringify({
                type: 'error',
                error: `Generation failed: ${error.message}`
            }));
        }
    }
}

/**
 * MCP Channel endpoint for BitNet to access Vectorizer tools
 */
app.post('/api/mcp', async (req, res) => {
    try {
        const { tool, args } = req.body;
        
        if (!tool) {
            return res.status(400).json({ error: 'Tool name is required' });
        }

        if (!mcpChannel) {
            return res.status(503).json({ error: 'MCP Channel not initialized' });
        }

        console.log(`🔧 BitNet MCP Request: ${tool}`, args);
        
        const result = await mcpChannel.callTool(tool, args || {});
        
        res.json({
            tool: tool,
            args: args,
            result: result,
            timestamp: new Date().toISOString()
        });

    } catch (error) {
        console.error('MCP Channel error:', error);
        res.status(500).json({ 
            error: 'MCP Channel error',
            message: error.message 
        });
    }
});

/**
 * Get available MCP tools
 */
app.get('/api/mcp/tools', (req, res) => {
    if (!mcpChannel) {
        return res.status(503).json({ error: 'MCP Channel not initialized' });
    }

    const tools = Object.keys(mcpChannel.tools).map(tool => ({
        name: tool,
        description: getToolDescription(tool)
    }));

    res.json({
        tools: tools,
        timestamp: new Date().toISOString()
    });
});

/**
 * Get tool descriptions
 */
function getToolDescription(tool) {
    const descriptions = {
        'list_collections': 'List all available collections in the Vectorizer',
        'search_vectors': 'Search for vectors in a collection using semantic similarity',
        'get_collection_info': 'Get detailed information about a specific collection',
        'insert_text': 'Insert new text into a collection for indexing',
        'delete_vectors': 'Delete specific vectors from a collection'
    };
    return descriptions[tool] || 'No description available';
}

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
        try {
            await startBitNetServer();
            console.log('✅ BitNet FastAPI server started successfully!');
        } catch (error) {
            console.error('❌ Failed to start BitNet server:', error.message);
            console.log('⚠️ Continuing without BitNet server...');
        }
        
        // Connect to Vectorizer via HTTP (assuming it's already running)
        // Try to connect to Vectorizer (optional)
        try {
            await connectToVectorizerHTTP();
        } catch (error) {
            console.warn('⚠️ Vectorizer not available:', error.message);
        }

        // Try to ensure collection exists (optional)
        try {
            await ensureCollectionExists();
        } catch (error) {
            console.warn('⚠️ Could not ensure collection exists:', error.message);
        }

        // Try to add sample knowledge (optional)
        try {
            await addSampleKnowledge();
        } catch (error) {
            console.warn('⚠️ Could not add sample knowledge:', error.message);
        }
        
        // Start HTTP server with WebSocket support
        server.listen(PORT, '0.0.0.0', () => {
            console.log(`🌐 Chat server with WebSocket running on http://localhost:${PORT}`);
            console.log(`🔌 WebSocket endpoint available at ws://localhost:${PORT}`);
            console.log(`📁 Model path: ${MODEL_PATH}`);
            console.log(`🔗 Vectorizer HTTP: http://localhost:${VECTORIZER_CONFIG.port}`);
            console.log(`🤖 BitNet FastAPI (Nova): http://localhost:${BITNET_CONFIG.port}`);
            console.log(`📚 Collection: ${VECTORIZER_CONFIG.collection}`);
            console.log(`🚀 All services ready! Open http://localhost:${PORT} to start chatting.`);
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
    
    // HTTP connections don't need explicit closing
    
    process.exit(0);
});

process.on('SIGTERM', async () => {
    console.log('\n🛑 Shutting down gracefully...');
    
    if (bitnetProcess) {
        console.log('🛑 Stopping BitNet server...');
        bitnetProcess.kill();
    }
    
    // HTTP connections don't need explicit closing
    
    process.exit(0);
});

// Start the server
initialize();

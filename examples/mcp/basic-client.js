#!/usr/bin/env node

/**
 * Basic MCP Client Example
 * 
 * This example demonstrates how to connect to Vectorizer's MCP server
 * and perform basic operations like searching vectors and managing collections.
 */

const WebSocket = require('ws');
const readline = require('readline');

class VectorizerMCPClient {
    constructor(url = 'ws://127.0.0.1:15003/mcp') {
        this.url = url;
        this.ws = null;
        this.messageId = 0;
        this.pendingRequests = new Map();
        this.connected = false;
    }

    async connect() {
        return new Promise((resolve, reject) => {
            console.log(`🔌 Connecting to MCP server at ${this.url}...`);
            
            this.ws = new WebSocket(this.url);
            
            this.ws.on('open', () => {
                console.log('✅ Connected to MCP server');
                this.connected = true;
                this.initialize();
                resolve();
            });
            
            this.ws.on('message', (data) => {
                this.handleMessage(data);
            });
            
            this.ws.on('error', (error) => {
                console.error('❌ WebSocket error:', error);
                reject(error);
            });
            
            this.ws.on('close', () => {
                console.log('🔌 Connection closed');
                this.connected = false;
            });
            
            // Connection timeout
            setTimeout(() => {
                if (!this.connected) {
                    reject(new Error('Connection timeout'));
                }
            }, 10000);
        });
    }

    async initialize() {
        try {
            const response = await this.sendRequest('initialize', {
                protocol_version: '2024-11-05',
                capabilities: {
                    tools: {}
                },
                client_info: {
                    name: 'Basic MCP Client',
                    version: '1.0.0'
                }
            });
            
            console.log('🚀 MCP initialization successful');
            console.log('Server info:', response.serverInfo);
        } catch (error) {
            console.error('❌ MCP initialization failed:', error);
        }
    }

    async sendRequest(method, params = {}) {
        if (!this.connected) {
            throw new Error('Not connected to MCP server');
        }

        const id = ++this.messageId;
        const message = {
            jsonrpc: '2.0',
            id: id,
            method: method,
            params: params
        };

        return new Promise((resolve, reject) => {
            this.pendingRequests.set(id, { resolve, reject });
            
            this.ws.send(JSON.stringify(message));
            
            // Request timeout
            setTimeout(() => {
                if (this.pendingRequests.has(id)) {
                    this.pendingRequests.delete(id);
                    reject(new Error('Request timeout'));
                }
            }, 30000);
        });
    }

    handleMessage(data) {
        try {
            const response = JSON.parse(data.toString());
            
            if (response.id && this.pendingRequests.has(response.id)) {
                const { resolve, reject } = this.pendingRequests.get(response.id);
                this.pendingRequests.delete(response.id);
                
                if (response.error) {
                    reject(new Error(response.error.message));
                } else {
                    resolve(response.result);
                }
            } else {
                // Handle notifications or other messages
                console.log('📨 Received message:', response);
            }
        } catch (error) {
            console.error('❌ Failed to parse message:', error);
        }
    }

    async callTool(toolName, arguments) {
        return this.sendRequest('tools/call', {
            name: toolName,
            arguments: arguments
        });
    }

    async listTools() {
        return this.sendRequest('tools/list');
    }

    async ping() {
        return this.sendRequest('ping');
    }

    // Convenience methods for common operations
    async searchVectors(collection, query, limit = 10) {
        return this.callTool('search_vectors', { collection, query, limit });
    }

    async listCollections() {
        return this.callTool('list_collections', {});
    }

    async getCollectionInfo(collection) {
        return this.callTool('get_collection_info', { collection });
    }

    async embedText(text) {
        return this.callTool('embed_text', { text });
    }

    async createCollection(name, dimension = 384, metric = 'cosine') {
        return this.callTool('create_collection', { name, dimension, metric });
    }

    async insertVectors(collection, vectors) {
        return this.callTool('insert_vectors', { collection, vectors });
    }

    async getDatabaseStats() {
        return this.callTool('get_database_stats', {});
    }

    disconnect() {
        if (this.ws) {
            this.ws.close();
        }
    }
}

// Interactive CLI
class MCPCLI {
    constructor(client) {
        this.client = client;
        this.rl = readline.createInterface({
            input: process.stdin,
            output: process.stdout,
            prompt: 'mcp> '
        });
    }

    start() {
        console.log('\n🎯 Vectorizer MCP Client CLI');
        console.log('Type "help" for available commands or "exit" to quit\n');
        
        this.rl.prompt();
        
        this.rl.on('line', async (line) => {
            const input = line.trim();
            
            if (input === 'exit') {
                this.client.disconnect();
                this.rl.close();
                return;
            }
            
            if (input === 'help') {
                this.showHelp();
                this.rl.prompt();
                return;
            }
            
            try {
                await this.handleCommand(input);
            } catch (error) {
                console.error('❌ Error:', error.message);
            }
            
            this.rl.prompt();
        });
        
        this.rl.on('close', () => {
            console.log('\n👋 Goodbye!');
            process.exit(0);
        });
    }

    showHelp() {
        console.log('\n📚 Available Commands:');
        console.log('  ping                           - Test connection');
        console.log('  tools                          - List available tools');
        console.log('  collections                    - List all collections');
        console.log('  info <collection>              - Get collection info');
        console.log('  search <collection> <query>    - Search vectors');
        console.log('  embed <text>                   - Generate embedding');
        console.log('  create <name> [dim] [metric]    - Create collection');
        console.log('  stats                          - Database statistics');
        console.log('  help                           - Show this help');
        console.log('  exit                           - Quit\n');
    }

    async handleCommand(input) {
        const parts = input.split(' ');
        const command = parts[0];
        
        switch (command) {
            case 'ping':
                const pong = await this.client.ping();
                console.log('🏓 Pong:', pong);
                break;
                
            case 'tools':
                const tools = await this.client.listTools();
                console.log('🛠️  Available tools:');
                tools.tools.forEach(tool => {
                    console.log(`  - ${tool.name}: ${tool.description}`);
                });
                break;
                
            case 'collections':
                const collections = await this.client.listCollections();
                console.log('📁 Collections:');
                collections.collections.forEach(col => {
                    console.log(`  - ${col.name}: ${col.vector_count} vectors, ${col.dimension}D`);
                });
                break;
                
            case 'info':
                if (parts.length < 2) {
                    console.log('❌ Usage: info <collection>');
                    return;
                }
                const info = await this.client.getCollectionInfo(parts[1]);
                console.log('📊 Collection info:', JSON.stringify(info, null, 2));
                break;
                
            case 'search':
                if (parts.length < 3) {
                    console.log('❌ Usage: search <collection> <query>');
                    return;
                }
                const collection = parts[1];
                const query = parts.slice(2).join(' ');
                const results = await this.client.searchVectors(collection, query);
                console.log('🔍 Search results:');
                results.results.forEach((result, index) => {
                    console.log(`  ${index + 1}. ${result.id} (score: ${result.score.toFixed(3)})`);
                    if (result.payload) {
                        console.log(`     Payload: ${JSON.stringify(result.payload)}`);
                    }
                });
                break;
                
            case 'embed':
                if (parts.length < 2) {
                    console.log('❌ Usage: embed <text>');
                    return;
                }
                const text = parts.slice(1).join(' ');
                const embedding = await this.client.embedText(text);
                console.log(`📝 Embedding for "${text}":`);
                console.log(`  Dimension: ${embedding.dimension}`);
                console.log(`  Provider: ${embedding.provider}`);
                console.log(`  Vector: [${embedding.embedding.slice(0, 5).join(', ')}, ...]`);
                break;
                
            case 'create':
                if (parts.length < 2) {
                    console.log('❌ Usage: create <name> [dimension] [metric]');
                    return;
                }
                const name = parts[1];
                const dimension = parts[2] ? parseInt(parts[2]) : 384;
                const metric = parts[3] || 'cosine';
                const result = await this.client.createCollection(name, dimension, metric);
                console.log('✅ Collection created:', result);
                break;
                
            case 'stats':
                const stats = await this.client.getDatabaseStats();
                console.log('📊 Database statistics:');
                console.log(`  Total collections: ${stats.total_collections}`);
                console.log(`  Total vectors: ${stats.total_vectors}`);
                console.log(`  Memory estimate: ${(stats.total_memory_estimate_bytes / 1024 / 1024).toFixed(2)} MB`);
                break;
                
            default:
                console.log('❌ Unknown command. Type "help" for available commands.');
        }
    }
}

// Main execution
async function main() {
    const client = new VectorizerMCPClient();
    
    try {
        await client.connect();
        
        // Run interactive CLI
        const cli = new MCPCLI(client);
        cli.start();
        
    } catch (error) {
        console.error('❌ Failed to connect to MCP server:', error.message);
        console.log('\n💡 Make sure the Vectorizer server is running:');
        console.log('   cargo run --bin vectorizer-server --features full');
        process.exit(1);
    }
}

// Handle graceful shutdown
process.on('SIGINT', () => {
    console.log('\n👋 Shutting down...');
    process.exit(0);
});

// Run if this file is executed directly
if (require.main === module) {
    main().catch(console.error);
}

module.exports = { VectorizerMCPClient };

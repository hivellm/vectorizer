#!/usr/bin/env python3

"""
Cursor IDE Integration Example
==============================

This example demonstrates how to integrate Vectorizer's MCP server with Cursor IDE
for semantic code search and documentation retrieval.

Features:
- Real-time code indexing
- Semantic search across codebase
- Documentation retrieval
- Code similarity detection
- Intelligent code suggestions
"""

import websocket
import json
import threading
import time
import os
import sys
from typing import Dict, List, Optional, Any
from dataclasses import dataclass
from pathlib import Path

@dataclass
class CodeSnippet:
    """Represents a code snippet with metadata."""
    id: str
    content: str
    file_path: str
    line_start: int
    line_end: int
    language: str
    function_name: Optional[str] = None
    class_name: Optional[str] = None
    docstring: Optional[str] = None

class VectorizerMCPClient:
    """MCP client for Vectorizer integration."""
    
    def __init__(self, url: str = "ws://127.0.0.1:15003/mcp"):
        self.url = url
        self.ws = None
        self.message_id = 0
        self.pending_requests = {}
        self.connected = False
        self.collection_name = "code_snippets"
        
    def connect(self) -> bool:
        """Connect to the MCP server."""
        try:
            print(f"🔌 Connecting to MCP server at {self.url}...")
            
            self.ws = websocket.WebSocketApp(
                self.url,
                on_open=self._on_open,
                on_message=self._on_message,
                on_error=self._on_error,
                on_close=self._on_close
            )
            
            # Run in separate thread
            wst = threading.Thread(target=self.ws.run_forever)
            wst.daemon = True
            wst.start()
            
            # Wait for connection
            timeout = 10
            start_time = time.time()
            while not self.connected and (time.time() - start_time) < timeout:
                time.sleep(0.1)
            
            if self.connected:
                self._initialize()
                return True
            else:
                print("❌ Connection timeout")
                return False
                
        except Exception as e:
            print(f"❌ Connection failed: {e}")
            return False
    
    def _on_open(self, ws):
        """Handle WebSocket connection open."""
        print("✅ Connected to MCP server")
        self.connected = True
    
    def _on_message(self, ws, message):
        """Handle incoming WebSocket messages."""
        try:
            response = json.loads(message)
            
            if 'id' in response and response['id'] in self.pending_requests:
                future = self.pending_requests[response['id']]
                del self.pending_requests[response['id']]
                
                if 'error' in response:
                    future['error'] = response['error']
                else:
                    future['result'] = response.get('result')
                
                future['event'].set()
            else:
                # Handle notifications
                print(f"📨 Notification: {response}")
                
        except json.JSONDecodeError as e:
            print(f"❌ Failed to parse message: {e}")
    
    def _on_error(self, ws, error):
        """Handle WebSocket errors."""
        print(f"❌ WebSocket error: {error}")
    
    def _on_close(self, ws, close_status_code, close_msg):
        """Handle WebSocket connection close."""
        print("🔌 Connection closed")
        self.connected = False
    
    def _initialize(self):
        """Initialize MCP connection."""
        try:
            response = self._send_request('initialize', {
                'protocol_version': '2024-11-05',
                'capabilities': {
                    'tools': {}
                },
                'client_info': {
                    'name': 'Cursor IDE Integration',
                    'version': '1.0.0'
                }
            })
            
            print("🚀 MCP initialization successful")
            print(f"Server: {response.get('serverInfo', {}).get('name', 'Unknown')}")
            
        except Exception as e:
            print(f"❌ MCP initialization failed: {e}")
    
    def _send_request(self, method: str, params: Dict = None) -> Any:
        """Send a request to the MCP server."""
        if not self.connected:
            raise Exception("Not connected to MCP server")
        
        self.message_id += 1
        message_id = self.message_id
        
        message = {
            'jsonrpc': '2.0',
            'id': message_id,
            'method': method,
            'params': params or {}
        }
        
        # Create future for response
        future = {
            'event': threading.Event(),
            'result': None,
            'error': None
        }
        self.pending_requests[message_id] = future
        
        # Send message
        self.ws.send(json.dumps(message))
        
        # Wait for response
        if future['event'].wait(timeout=30):
            if future['error']:
                raise Exception(f"MCP error: {future['error']['message']}")
            return future['result']
        else:
            del self.pending_requests[message_id]
            raise Exception("Request timeout")
    
    def _call_tool(self, tool_name: str, arguments: Dict) -> Any:
        """Call an MCP tool."""
        return self._send_request('tools/call', {
            'name': tool_name,
            'arguments': arguments
        })
    
    def ensure_collection(self) -> bool:
        """Ensure the code snippets collection exists."""
        try:
            # Try to get collection info
            self._call_tool('get_collection_info', {'collection': self.collection_name})
            return True
        except:
            # Collection doesn't exist, create it
            try:
                print(f"📁 Creating collection '{self.collection_name}'...")
                self._call_tool('create_collection', {
                    'name': self.collection_name,
                    'dimension': 768,  # Higher dimension for code embeddings
                    'metric': 'cosine'
                })
                print(f"✅ Collection '{self.collection_name}' created")
                return True
            except Exception as e:
                print(f"❌ Failed to create collection: {e}")
                return False
    
    def index_code_snippet(self, snippet: CodeSnippet) -> bool:
        """Index a code snippet in the vector database."""
        try:
            # Generate embedding for the code content
            embedding_result = self._call_tool('embed_text', {'text': snippet.content})
            embedding = embedding_result['embedding']
            
            # Prepare vector data
            vector_data = {
                'id': snippet.id,
                'data': embedding,
                'payload': {
                    'file_path': snippet.file_path,
                    'line_start': snippet.line_start,
                    'line_end': snippet.line_end,
                    'language': snippet.language,
                    'function_name': snippet.function_name,
                    'class_name': snippet.class_name,
                    'docstring': snippet.docstring,
                    'content': snippet.content
                }
            }
            
            # Insert vector
            self._call_tool('insert_vectors', {
                'collection': self.collection_name,
                'vectors': [vector_data]
            })
            
            return True
            
        except Exception as e:
            print(f"❌ Failed to index snippet {snippet.id}: {e}")
            return False
    
    def search_code(self, query: str, limit: int = 10) -> List[Dict]:
        """Search for similar code snippets."""
        try:
            result = self._call_tool('search_vectors', {
                'collection': self.collection_name,
                'query': query,
                'limit': limit
            })
            
            return result.get('results', [])
            
        except Exception as e:
            print(f"❌ Search failed: {e}")
            return []
    
    def get_database_stats(self) -> Dict:
        """Get database statistics."""
        try:
            return self._call_tool('get_database_stats', {})
        except Exception as e:
            print(f"❌ Failed to get stats: {e}")
            return {}

class CursorCodeIndexer:
    """Indexes code files for Cursor IDE integration."""
    
    def __init__(self, mcp_client: VectorizerMCPClient):
        self.client = mcp_client
        self.supported_extensions = {
            '.py': 'python',
            '.js': 'javascript',
            '.ts': 'typescript',
            '.rs': 'rust',
            '.go': 'go',
            '.java': 'java',
            '.cpp': 'cpp',
            '.c': 'c',
            '.h': 'c',
            '.hpp': 'cpp',
            '.cs': 'csharp',
            '.php': 'php',
            '.rb': 'ruby',
            '.swift': 'swift',
            '.kt': 'kotlin',
            '.scala': 'scala'
        }
    
    def index_directory(self, directory: str, recursive: bool = True) -> int:
        """Index all code files in a directory."""
        indexed_count = 0
        directory_path = Path(directory)
        
        if not directory_path.exists():
            print(f"❌ Directory not found: {directory}")
            return 0
        
        print(f"📂 Indexing directory: {directory}")
        
        # Get all code files
        code_files = []
        if recursive:
            for ext in self.supported_extensions.keys():
                code_files.extend(directory_path.rglob(f"*{ext}"))
        else:
            for ext in self.supported_extensions.keys():
                code_files.extend(directory_path.glob(f"*{ext}"))
        
        print(f"📄 Found {len(code_files)} code files")
        
        # Index each file
        for file_path in code_files:
            try:
                indexed_count += self.index_file(str(file_path))
            except Exception as e:
                print(f"❌ Failed to index {file_path}: {e}")
        
        print(f"✅ Indexed {indexed_count} code snippets")
        return indexed_count
    
    def index_file(self, file_path: str) -> int:
        """Index a single code file."""
        try:
            file_path_obj = Path(file_path)
            extension = file_path_obj.suffix.lower()
            
            if extension not in self.supported_extensions:
                return 0
            
            language = self.supported_extensions[extension]
            
            with open(file_path, 'r', encoding='utf-8') as f:
                content = f.read()
            
            # Parse code into snippets
            snippets = self._parse_code_file(content, str(file_path), language)
            
            # Index each snippet
            indexed_count = 0
            for snippet in snippets:
                if self.client.index_code_snippet(snippet):
                    indexed_count += 1
            
            print(f"📝 Indexed {indexed_count} snippets from {file_path}")
            return indexed_count
            
        except Exception as e:
            print(f"❌ Failed to index file {file_path}: {e}")
            return 0
    
    def _parse_code_file(self, content: str, file_path: str, language: str) -> List[CodeSnippet]:
        """Parse a code file into snippets."""
        snippets = []
        lines = content.split('\n')
        
        # Simple parsing - split by functions/classes
        current_snippet = []
        current_line = 0
        snippet_id = 0
        
        for i, line in enumerate(lines):
            current_snippet.append(line)
            
            # Check for function/class boundaries
            if self._is_function_or_class_start(line, language):
                if len(current_snippet) > 1:  # Don't create empty snippets
                    snippet_id += 1
                    snippet_content = '\n'.join(current_snippet[:-1])
                    
                    snippet = CodeSnippet(
                        id=f"{Path(file_path).stem}_{snippet_id}",
                        content=snippet_content,
                        file_path=file_path,
                        line_start=current_line,
                        line_end=current_line + len(current_snippet) - 2,
                        language=language,
                        function_name=self._extract_function_name(current_snippet[-1], language),
                        class_name=self._extract_class_name(current_snippet[-1], language)
                    )
                    snippets.append(snippet)
                
                current_line = i
                current_snippet = [line]
        
        # Add remaining content as final snippet
        if current_snippet:
            snippet_id += 1
            snippet_content = '\n'.join(current_snippet)
            
            snippet = CodeSnippet(
                id=f"{Path(file_path).stem}_{snippet_id}",
                content=snippet_content,
                file_path=file_path,
                line_start=current_line,
                line_end=current_line + len(current_snippet) - 1,
                language=language
            )
            snippets.append(snippet)
        
        return snippets
    
    def _is_function_or_class_start(self, line: str, language: str) -> bool:
        """Check if line starts a function or class."""
        line = line.strip()
        
        if language == 'python':
            return (line.startswith('def ') or 
                   line.startswith('class ') or
                   line.startswith('async def '))
        elif language in ['javascript', 'typescript']:
            return (line.startswith('function ') or
                   'function(' in line or
                   '=>' in line or
                   line.startswith('class '))
        elif language == 'rust':
            return (line.startswith('fn ') or
                   line.startswith('impl ') or
                   line.startswith('struct ') or
                   line.startswith('enum '))
        elif language == 'go':
            return (line.startswith('func ') or
                   line.startswith('type '))
        else:
            # Generic detection
            return ('function' in line.lower() or
                   'class' in line.lower() or
                   'def ' in line.lower())
    
    def _extract_function_name(self, line: str, language: str) -> Optional[str]:
        """Extract function name from line."""
        line = line.strip()
        
        if language == 'python':
            if line.startswith('def '):
                return line.split('(')[0].replace('def ', '').strip()
        elif language in ['javascript', 'typescript']:
            if 'function' in line:
                parts = line.split('function')
                if len(parts) > 1:
                    return parts[1].split('(')[0].strip()
        elif language == 'rust':
            if line.startswith('fn '):
                return line.split('(')[0].replace('fn ', '').strip()
        elif language == 'go':
            if line.startswith('func '):
                return line.split('(')[0].replace('func ', '').strip()
        
        return None
    
    def _extract_class_name(self, line: str, language: str) -> Optional[str]:
        """Extract class name from line."""
        line = line.strip()
        
        if 'class ' in line.lower():
            parts = line.split('class')
            if len(parts) > 1:
                return parts[1].split('(')[0].split(':')[0].strip()
        
        return None

class CursorIDEIntegration:
    """Main integration class for Cursor IDE."""
    
    def __init__(self):
        self.client = VectorizerMCPClient()
        self.indexer = CursorCodeIndexer(self.client)
    
    def setup(self) -> bool:
        """Setup the integration."""
        print("🚀 Setting up Cursor IDE integration...")
        
        # Connect to MCP server
        if not self.client.connect():
            return False
        
        # Ensure collection exists
        if not self.client.ensure_collection():
            return False
        
        print("✅ Cursor IDE integration ready")
        return True
    
    def index_project(self, project_path: str) -> int:
        """Index an entire project."""
        print(f"📂 Indexing project: {project_path}")
        return self.indexer.index_directory(project_path, recursive=True)
    
    def search_code(self, query: str, limit: int = 10) -> List[Dict]:
        """Search for code snippets."""
        print(f"🔍 Searching for: {query}")
        results = self.client.search_code(query, limit)
        
        if results:
            print(f"📋 Found {len(results)} results:")
            for i, result in enumerate(results, 1):
                payload = result.get('payload', {})
                print(f"  {i}. {result['id']} (score: {result['score']:.3f})")
                print(f"     File: {payload.get('file_path', 'Unknown')}")
                print(f"     Lines: {payload.get('line_start', '?')}-{payload.get('line_end', '?')}")
                if payload.get('function_name'):
                    print(f"     Function: {payload['function_name']}")
                print()
        else:
            print("❌ No results found")
        
        return results
    
    def get_stats(self):
        """Get database statistics."""
        stats = self.client.get_database_stats()
        print("📊 Database Statistics:")
        print(f"  Collections: {stats.get('total_collections', 0)}")
        print(f"  Vectors: {stats.get('total_vectors', 0)}")
        print(f"  Memory: {stats.get('total_memory_estimate_bytes', 0) / 1024 / 1024:.2f} MB")
    
    def interactive_mode(self):
        """Run interactive mode."""
        print("\n🎯 Cursor IDE Integration - Interactive Mode")
        print("Type 'help' for commands or 'exit' to quit\n")
        
        while True:
            try:
                command = input("cursor> ").strip()
                
                if command == 'exit':
                    break
                elif command == 'help':
                    self._show_help()
                elif command.startswith('index '):
                    path = command[6:].strip()
                    self.index_project(path)
                elif command.startswith('search '):
                    query = command[7:].strip()
                    self.search_code(query)
                elif command == 'stats':
                    self.get_stats()
                else:
                    print("❌ Unknown command. Type 'help' for available commands.")
                    
            except KeyboardInterrupt:
                break
            except Exception as e:
                print(f"❌ Error: {e}")
        
        print("\n👋 Goodbye!")
    
    def _show_help(self):
        """Show help information."""
        print("\n📚 Available Commands:")
        print("  index <path>     - Index a project directory")
        print("  search <query>   - Search for code snippets")
        print("  stats            - Show database statistics")
        print("  help             - Show this help")
        print("  exit             - Quit\n")

def main():
    """Main function."""
    integration = CursorIDEIntegration()
    
    if not integration.setup():
        print("❌ Failed to setup integration")
        sys.exit(1)
    
    # Check command line arguments
    if len(sys.argv) > 1:
        command = sys.argv[1]
        
        if command == 'index' and len(sys.argv) > 2:
            project_path = sys.argv[2]
            integration.index_project(project_path)
        elif command == 'search' and len(sys.argv) > 2:
            query = ' '.join(sys.argv[2:])
            integration.search_code(query)
        elif command == 'stats':
            integration.get_stats()
        else:
            print("❌ Invalid command or arguments")
            print("Usage:")
            print("  python cursor-integration.py index <path>")
            print("  python cursor-integration.py search <query>")
            print("  python cursor-integration.py stats")
    else:
        # Run interactive mode
        integration.interactive_mode()

if __name__ == "__main__":
    main()

#!/usr/bin/env python3
"""
BitNet Server - Versão Final Corrigida
Servidor FastAPI com busca otimizada que funciona corretamente
"""

import asyncio
import json
import logging
import os
import subprocess
import tempfile
import time
from typing import List, Dict, Any, Optional
from datetime import datetime

import httpx
import uvicorn
import torch
from transformers import AutoTokenizer, AutoModelForCausalLM
from fastapi import FastAPI, HTTPException, WebSocket, WebSocketDisconnect
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import HTMLResponse
from pydantic import BaseModel

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)

# BitNet Model configuration
MODEL_NAME = "microsoft/bitnet-b1.58-2B-4T"
MAX_TOKENS = 512
TEMPERATURE = 0.7
TOP_P = 0.9

# Global BitNet variables
bitnet_model = None
bitnet_tokenizer = None
bitnet_loaded = False

# =============================================================================
# BITNET REAL FUNCTIONS
# =============================================================================

async def load_bitnet_model():
    """Load the BitNet model and tokenizer"""
    global bitnet_model, bitnet_tokenizer, bitnet_loaded
    
    try:
        logger.info("🔄 Loading BitNet REAL model and tokenizer...")
        
        # Check GPU availability
        gpu_available = False
        if torch.cuda.is_available():
            logger.info(f"🚀 GPU detected: {torch.cuda.get_device_name(0)}")
            gpu_available = True
        else:
            logger.info("💻 No GPU detected, using CPU")
        
        # Load tokenizer
        bitnet_tokenizer = AutoTokenizer.from_pretrained(MODEL_NAME)
        logger.info("✅ BitNet tokenizer loaded")
        
        # Load model
        if gpu_available:
            bitnet_model = AutoModelForCausalLM.from_pretrained(
                    MODEL_NAME,
                    torch_dtype=torch.bfloat16,
                    device_map="auto",
                    low_cpu_mem_usage=True,
                    trust_remote_code=True
                )
            bitnet_model = bitnet_model.cuda()
            logger.info("✅ BitNet model loaded on GPU")
        else:
            bitnet_model = AutoModelForCausalLM.from_pretrained(
                MODEL_NAME,
                torch_dtype=torch.float32,
                device_map="cpu",
                low_cpu_mem_usage=True
            )
            logger.info("✅ BitNet model loaded on CPU")
        
        bitnet_model.eval()
        bitnet_loaded = True
        logger.info("🎉 BitNet REAL model ready for inference!")
        
    except Exception as e:
        logger.error(f"❌ Failed to load BitNet model: {e}")
        raise

async def generate_bitnet_response(prompt: str) -> str:
    """
    Gera resposta usando BitNet REAL
    """
    global bitnet_model, bitnet_tokenizer, bitnet_loaded
    
    if not bitnet_loaded:
        try:
            await load_bitnet_model()
    except Exception as e:
            return f"Erro ao carregar BitNet REAL: {e}"
    
    try:
        logger.info(f"🤖 Generating response with BitNet REAL for prompt: {prompt[:100]}...")
        
        # Format prompt for BitNet
        formatted_prompt = f"""You are BitNet, a helpful AI assistant. Answer the user's question clearly and concisely.

Question: {prompt}

Answer:"""
        
        # Tokenize input
        inputs = bitnet_tokenizer(formatted_prompt, return_tensors="pt")
        
        # Move to same device as model
        device = next(bitnet_model.parameters()).device
        inputs = {k: v.to(device) for k, v in inputs.items()}
        
        # Generate response
        with torch.no_grad():
            outputs = bitnet_model.generate(
                **inputs,
                max_new_tokens=200,
                temperature=TEMPERATURE,
                top_p=TOP_P,
                do_sample=True,
                pad_token_id=bitnet_tokenizer.eos_token_id,
                eos_token_id=bitnet_tokenizer.eos_token_id,
                repetition_penalty=1.1,
                no_repeat_ngram_size=2,
                use_cache=True
            )
        
        # Decode response
        input_length = inputs['input_ids'].shape[1]
        response_tokens = outputs[0][input_length:]
        response = bitnet_tokenizer.decode(response_tokens, skip_special_tokens=True)
        
        logger.info(f"✅ BitNet REAL response generated: {response[:100]}...")
        return response.strip()
            
    except Exception as e:
        logger.error(f"❌ BitNet REAL generation failed: {e}")
        return f"Erro ao gerar resposta com BitNet REAL: {e}"

# =============================================================================
# MODELS
# =============================================================================

class ChatRequest(BaseModel):
    message: str
    history: List[Dict[str, str]] = []

class ChatResponse(BaseModel):
    response: str
    context_used: bool
    search_results: int
    processing_time: float

class HealthResponse(BaseModel):
    status: str
    vectorizer_connected: bool
    timestamp: str

# =============================================================================
# VECTORIZER CLIENT
# =============================================================================

class VectorizerClient:
    """Cliente otimizado para o Vectorizer"""
    
    def __init__(self, base_url: str = "http://localhost:15002"):
        self.base_url = base_url
        self.client = httpx.AsyncClient(timeout=30.0)
        self._collections_cache = None
        self._cache_timestamp = 0
        self._cache_duration = 60  # 1 minute cache
    
    async def health_check(self) -> bool:
        """Verifica se o Vectorizer está funcionando"""
        try:
            response = await self.client.get(f"{self.base_url}/health")
            return response.status_code == 200
        except:
            return False

    async def list_collections(self) -> List[Dict[str, Any]]:
        """Lista coleções com cache"""
        current_time = time.time()
        
        # Use cache if still valid
        if (self._collections_cache and 
            current_time - self._cache_timestamp < self._cache_duration):
            return self._collections_cache
        
        try:
            response = await self.client.get(f"{self.base_url}/collections")
            response.raise_for_status()
            result = response.json()
            collections = result.get("collections", [])
            
            # Update cache
            self._collections_cache = collections
            self._cache_timestamp = current_time
            
            return collections
        except Exception as e:
            logger.warning(f"Failed to list collections: {e}")
            return []
    
    async def search_collection(self, collection: str, query: str, limit: int = 3) -> List[Dict[str, Any]]:
        """Busca em uma coleção específica"""
        try:
            response = await self.client.post(
                f"{self.base_url}/collections/{collection}/search/text",
                json={"query": query, "limit": limit},
                timeout=10.0
            )
            
            if response.status_code == 200:
                result = response.json()
                return result.get("results", [])
        else:
                logger.warning(f"Search failed for {collection}: {response.status_code}")
                return []
        
    except Exception as e:
            logger.warning(f"Error searching {collection}: {e}")
            return []
    
    async def intelligent_search(self, query: str, max_results: int = 5) -> List[Dict[str, Any]]:
        """Busca inteligente otimizada - VERSÃO FINAL CORRIGIDA"""
        try:
            logger.info(f"Starting intelligent search for: '{query}'")
            
            # Get collections
            collections = await self.list_collections()
            collection_names = [col["name"] for col in collections if col.get("document_count", 0) > 0]
            
            logger.info(f"Available collections: {len(collection_names)}")
            
            # Check if query is about vectorizer - CORRIGIDO
            query_lower = query.lower()
            is_vectorizer_query = any(keyword in query_lower for keyword in [
                'vectorizer', 'vector', 'embedding', 'search', 'rust', 'cargo'
            ])
            
            logger.info(f"Vectorizer query detected: {is_vectorizer_query}")
            
            all_results = []
            
            if is_vectorizer_query:
                # Force search only in vectorizer collections
                vectorizer_collections = [col for col in collection_names if 'vectorizer' in col.lower()]
                logger.info(f"Vectorizer collections found: {vectorizer_collections}")
                
                if not vectorizer_collections:
                    logger.warning("No vectorizer collections found!")
                    return []
                
                for collection in vectorizer_collections[:3]:  # Top 3 vectorizer collections
                    try:
                        results = await self.search_collection(collection, query, limit=3)
                        logger.info(f"Searched {collection}: {len(results)} results")
                        
                        for item in results:
                            if isinstance(item, dict):
                                # Extract content safely
                                payload = item.get("payload", {})
                                content = payload.get("content", "") if isinstance(payload, dict) else ""
                                
                                # Handle encoding
                                try:
                                    content = content.encode('utf-8', errors='ignore').decode('utf-8')
                                except:
                                    content = str(content)
                                
                                processed_item = {
                                    "content": content,
                                    "score": item.get("score", 0.0),
                                    "collection": collection,
                                    "doc_id": item.get("id", ""),
                                    "metadata": payload if isinstance(payload, dict) else {}
                                }
                                all_results.append(processed_item)
                        
                except Exception as e:
                        logger.warning(f"Failed to search vectorizer collection {collection}: {e}")
                        continue
            else:
                # Use normal prioritization for other queries
                prioritized_collections = self._prioritize_collections(query, collection_names)
                logger.info(f"Normal query, prioritized collections: {prioritized_collections[:5]}")
                
                for collection in prioritized_collections[:5]:  # Top 5 collections
                    try:
                        results = await self.search_collection(collection, query, limit=2)
                        
                        for item in results:
                            if isinstance(item, dict):
                                # Extract content safely
                                payload = item.get("payload", {})
                                content = payload.get("content", "") if isinstance(payload, dict) else ""
                                
                                # Handle encoding
                                try:
                                    content = content.encode('utf-8', errors='ignore').decode('utf-8')
                                except:
                                    content = str(content)
                                
                                processed_item = {
                                    "content": content,
                                    "score": item.get("score", 0.0),
                                    "collection": collection,
                                    "doc_id": item.get("id", ""),
                                    "metadata": payload if isinstance(payload, dict) else {}
                                }
                                all_results.append(processed_item)
                                
                    except Exception as e:
                        logger.warning(f"Failed to search {collection}: {e}")
                continue

            # Sort by score and return top results
            all_results.sort(key=lambda x: x.get("score", 0.0), reverse=True)
            final_results = all_results[:max_results]
            
            logger.info(f"Found {len(final_results)} relevant results")
            return final_results
            
        except Exception as e:
            logger.error(f"Intelligent search failed: {e}")
            return []
    
    def _prioritize_collections(self, query: str, collections: List[str]) -> List[str]:
        """Prioritiza coleções baseado na query"""
        query_lower = query.lower()
        prioritized = []
        
        # 1. Direct keyword matches (highest priority)
        query_keywords = query_lower.split()
        
        for keyword in query_keywords:
            for col in collections:
                if keyword in col.lower() and col not in prioritized:
                    prioritized.append(col)
        
        # 2. Vectorizer-specific collections (high priority)
        vectorizer_patterns = ['vectorizer', 'rust', 'cargo', 'embedding', 'search']
        for pattern in vectorizer_patterns:
            for col in collections:
                if pattern in col.lower() and col not in prioritized:
                    prioritized.append(col)
        
        # 3. Documentation collections (medium priority)
        doc_patterns = ['docs', 'guide', 'manual', 'api', 'reference']
        for pattern in doc_patterns:
            for col in collections:
                if pattern in col.lower() and col not in prioritized:
                    prioritized.append(col)
        
        # 4. Source code collections (lower priority)
        source_patterns = ['source', 'src', 'code']
        for pattern in source_patterns:
            for col in collections:
                if pattern in col.lower() and col not in prioritized:
                    prioritized.append(col)
        
        # 5. Add remaining collections (lowest priority)
        for col in collections:
            if col not in prioritized:
                prioritized.append(col)
        
        return prioritized

# =============================================================================
# BITNET SERVER
# =============================================================================

class BitNetServer:
    """Servidor BitNet principal"""
    
    def __init__(self):
        self.app = FastAPI(
            title="BitNet Server",
            description="Servidor BitNet com busca otimizada do Vectorizer e BitNet REAL",
            version="2.0.0"
        )
        
        # Initialize Vectorizer client
        self.vectorizer = VectorizerClient()
        
        # Initialize BitNet model
        self._init_bitnet()
        
        # Setup CORS
        self.app.add_middleware(
            CORSMiddleware,
            allow_origins=["*"],
            allow_credentials=True,
            allow_methods=["*"],
            allow_headers=["*"],
        )
        
        # Setup routes
        self._setup_routes()
        
        # WebSocket connections
        self.active_connections: List[WebSocket] = []
    
    def _init_bitnet(self):
        """Initialize BitNet model"""
        try:
            logger.info("🔄 Initializing BitNet REAL model...")
            # BitNet will be loaded on first request
            logger.info("✅ BitNet initialization scheduled")
            except Exception as e:
            logger.error(f"❌ BitNet initialization failed: {e}")
    
    def _setup_routes(self):
        """Configura as rotas da API"""
        
        @self.app.get("/", response_class=HTMLResponse)
        async def root():
            return """
            <!DOCTYPE html>
            <html>
            <head>
                <title>BitNet Server v2.0</title>
                <style>
                    body { font-family: Arial, sans-serif; margin: 40px; background: #f5f5f5; }
                    .container { max-width: 800px; margin: 0 auto; background: white; padding: 30px; border-radius: 10px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }
                    h1 { color: #333; text-align: center; }
                    .status { padding: 15px; margin: 20px 0; border-radius: 5px; }
                    .healthy { background: #d4edda; color: #155724; border: 1px solid #c3e6cb; }
                    .unhealthy { background: #f8d7da; color: #721c24; border: 1px solid #f5c6cb; }
                    .endpoints { margin-top: 30px; }
                    .endpoint { background: #f8f9fa; padding: 15px; margin: 10px 0; border-radius: 5px; border-left: 4px solid #007bff; }
                    code { background: #e9ecef; padding: 2px 6px; border-radius: 3px; }
                </style>
            </head>
            <body>
                <div class="container">
                    <h1>BitNet Server v2.0 - FINAL</h1>
                    <div id="status" class="status">
                        <strong>Status:</strong> <span id="status-text">Checking...</span>
                    </div>
                    <div class="endpoints">
                        <h3>Available Endpoints:</h3>
                        <div class="endpoint">
                            <strong>POST /api/chat</strong> - Chat with BitNet<br>
                            <code>{"message": "your question", "history": []}</code>
                        </div>
                        <div class="endpoint">
                            <strong>GET /api/health</strong> - Health check
                        </div>
                        <div class="endpoint">
                            <strong>WebSocket /ws</strong> - Real-time chat
                        </div>
                    </div>
                </div>
                <script>
                    fetch('/api/health')
                        .then(response => response.json())
                        .then(data => {
                            const statusEl = document.getElementById('status-text');
                            const statusDiv = document.getElementById('status');
                            if (data.status === 'healthy') {
                                statusEl.textContent = 'Healthy - All systems operational';
                                statusDiv.className = 'status healthy';
                            } else {
                                statusEl.textContent = 'Unhealthy - Check logs';
                                statusDiv.className = 'status unhealthy';
                            }
                        })
                        .catch(error => {
                            document.getElementById('status-text').textContent = 'Error checking status';
                            document.getElementById('status').className = 'status unhealthy';
                        });
                </script>
            </body>
            </html>
            """
        
        @self.app.get("/api/health", response_model=HealthResponse)
        async def health():
            vectorizer_ok = await self.vectorizer.health_check()
            return HealthResponse(
                status="healthy" if vectorizer_ok else "unhealthy",
                vectorizer_connected=vectorizer_ok,
                timestamp=datetime.now().isoformat()
            )
        
        @self.app.post("/api/chat", response_model=ChatResponse)
        async def chat(request: ChatRequest):
            start_time = time.time()
            
            try:
                logger.info(f"Chat request: '{request.message}'")
                
                # Perform intelligent search
                search_results = await self.vectorizer.intelligent_search(request.message, max_results=5)
                
                # Build context
                context = self._build_context(search_results)
                
                # Generate response (simplified for now)
                response = await self._generate_response(request.message, context, search_results)
                
                processing_time = time.time() - start_time
                
                logger.info(f"Response generated in {processing_time:.2f}s")
                
                return ChatResponse(
                    response=response,
                    context_used=len(search_results) > 0,
                    search_results=len(search_results),
                    processing_time=processing_time
                )
                    
            except Exception as e:
                logger.error(f"Chat error: {e}")
                raise HTTPException(status_code=500, detail=str(e))
        
        @self.app.websocket("/ws")
        async def websocket_endpoint(websocket: WebSocket):
            await websocket.accept()
            self.active_connections.append(websocket)
            
            try:
                while True:
                    data = await websocket.receive_text()
                    message_data = json.loads(data)
                    
                    if message_data.get("type") == "chat":
                        message = message_data.get("message", "")
                        history = message_data.get("history", [])
                        
                        # Send status updates
                        await websocket.send_text(json.dumps({
                            "type": "status",
                            "status": "searching",
                            "message": "Searching knowledge base..."
                        }))
                        
                        # Perform search
                        search_results = await self.vectorizer.intelligent_search(message, max_results=5)
                        
                        await websocket.send_text(json.dumps({
                            "type": "status",
                            "status": "generating",
                            "message": f"Found {len(search_results)} relevant results, generating response..."
                        }))
                        
                        # Generate response
                        context = self._build_context(search_results)
                        response = await self._generate_response(message, context, search_results)
                        
                        # Send response
                        await websocket.send_text(json.dumps({
                            "type": "response",
                            "response": response,
                            "search_results": len(search_results)
                        }))
                        
            except WebSocketDisconnect:
                self.active_connections.remove(websocket)
    except Exception as e:
                logger.error(f"WebSocket error: {e}")
                await websocket.send_text(json.dumps({
                    "type": "error",
                    "error": str(e)
                }))
    
    def _build_context(self, search_results: List[Dict[str, Any]]) -> str:
        """Constrói contexto a partir dos resultados da busca"""
        if not search_results:
            return ""
        
        context_parts = []
        for result in search_results:
            content = result.get('content', '')
            score = result.get('score', 0.0)
            collection = result.get('collection', 'unknown')
            
            # Filter by score threshold
            if score >= -0.1:  # BM25 scores can be negative
                # Truncate content
                if len(content) > 500:
                    content = content[:500] + "..."
                
                formatted_content = f"[{collection}] (score: {score:.3f})\n{content}"
                context_parts.append(formatted_content)
        
        return "\n\n".join(context_parts)
    
    async def _generate_response(self, message: str, context: str, search_results: List[Dict[str, Any]]) -> str:
        """Gera resposta usando BitNet real"""
        
        if not search_results:
            return f"Desculpe, não encontrei informações relevantes sobre '{message}' na base de conhecimento."
        
        # Prepare prompt for BitNet
        prompt = f"""Baseado no contexto encontrado, responda à pergunta do usuário de forma clara e informativa.

PERGUNTA: {message}

CONTEXTO ENCONTRADO:
{context}

RESPOSTA:"""
        
        # Try to use BitNet model
        try:
            logger.info("🤖 Attempting to call BitNet model...")
            bitnet_response = await self._call_bitnet_model(prompt)
            logger.info(f"🤖 BitNet response: {bitnet_response[:100]}...")
            if bitnet_response and not bitnet_response.startswith("Erro") and not bitnet_response.startswith("Modelo"):
                logger.info("✅ Using BitNet response")
                return bitnet_response
            else:
                logger.warning("⚠️ BitNet response not suitable, using fallback")
        except Exception as e:
            logger.warning(f"⚠️ BitNet model failed: {e}")
        
        # Fallback to formatted response
        collections_found = set()
        for result in search_results:
            collections_found.add(result.get('collection', 'unknown'))
        
        response_parts = []
        response_parts.append(f"Encontrei informações relevantes sobre '{message}' em {len(collections_found)} coleções:")
        
        for collection in sorted(collections_found):
            response_parts.append(f"- {collection}")
        
        response_parts.append("\nBaseado no contexto encontrado:")
        
        # Add context summary
        if context:
            # Take first 300 chars of context
            context_preview = context[:300] + "..." if len(context) > 300 else context
            response_parts.append(context_preview)
        
        return "\n".join(response_parts)
    
    async def _call_bitnet_model(self, prompt: str) -> str:
        """Chama o modelo BitNet REAL para gerar resposta"""
        try:
            logger.info("🤖 Calling BitNet REAL model...")
            response = await generate_bitnet_response(prompt)
            logger.info(f"🤖 BitNet REAL response: {response[:100]}...")
            return response
        
    except Exception as e:
            logger.error(f"Error calling BitNet REAL model: {e}")
            return "Erro ao chamar modelo BitNet REAL."

# =============================================================================
# MAIN
# =============================================================================

def main():
    """Função principal"""
    logger.info("Starting BitNet Server v2.0 - FINAL")
    
    # Create server
    server = BitNetServer()
    
    # Run server
    uvicorn.run(
        server.app,
        host="0.0.0.0",
        port=15006,  # Different port to avoid conflicts
        log_level="info"
    )

if __name__ == "__main__":
    main()

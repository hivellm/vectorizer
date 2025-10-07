#!/usr/bin/env python3
"""
BitNet Server com Vectorizer - VERSÃO FINAL CORRIGIDA
Servidor FastAPI que integra busca inteligente do Vectorizer com BitNet REAL
"""

import asyncio
import json
import logging
import time
from pathlib import Path
from typing import Any, Dict, List, Optional
import torch
from transformers import AutoTokenizer, AutoModelForCausalLM

import httpx
from fastapi import FastAPI, WebSocket, WebSocketDisconnect, HTTPException
from fastapi.responses import HTMLResponse
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel

# Disable PyTorch compilation to avoid C++ compiler issues
import os
os.environ['TORCH_COMPILE_DISABLED'] = '1'
os.environ['TORCHDYNAMO_DISABLE'] = '1'

# =============================================================================
# CONFIGURAÇÃO
# =============================================================================

# BitNet Model configuration
MODEL_NAME = "microsoft/bitnet-b1.58-2B-4T"
MAX_TOKENS = 512
TEMPERATURE = 0.7
TOP_P = 0.9

# Global BitNet variables
bitnet_model = None
bitnet_tokenizer = None
bitnet_loaded = False

# Logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

# Request/Response models
class ChatMessage(BaseModel):
    role: str
    content: str

class GenerateRequest(BaseModel):
    messages: List[ChatMessage]
    context: Optional[str] = ""
    max_tokens: Optional[int] = MAX_TOKENS
    temperature: Optional[float] = TEMPERATURE
    top_p: Optional[float] = TOP_P

class GenerateResponse(BaseModel):
    response: str
    tokens_generated: int
    generation_time: float
    model_info: Dict[str, Any]
    
    model_config = {"protected_namespaces": ()}

class HealthResponse(BaseModel):
    status: str
    model_loaded: bool
    model_info: Dict[str, Any]
    
    model_config = {"protected_namespaces": ()}

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
        
        # Generate response with conservative parameters
        with torch.no_grad():
            outputs = bitnet_model.generate(
                **inputs,
                max_new_tokens=300,  # Increased for better responses
                temperature=0.5,     # Balanced temperature
                top_p=0.9,          # Conservative top_p
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
# VECTORIZER CLIENT
# =============================================================================

class VectorizerClient:
    """Cliente para comunicação com Vectorizer via REST API"""
    
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
        if (self._collections_cache is not None and 
            current_time - self._cache_timestamp < self._cache_duration):
            return self._collections_cache
        
        try:
            response = await self.client.get(f"{self.base_url}/collections")
            if response.status_code == 200:
                result = response.json()
                collections = result.get("collections", [])
                
                # Filter collections with documents
                collections_with_data = [
                    col for col in collections 
                    if col.get("document_count", 0) > 0
                ]
                
                self._collections_cache = collections_with_data
                self._cache_timestamp = current_time
                return collections_with_data
            else:
                logger.warning(f"Failed to list collections: {response.status_code}")
                return []
        except Exception as e:
            logger.warning(f"Error listing collections: {e}")
            return []

    async def search_collection(self, collection: str, query: str, limit: int = 5) -> List[Dict[str, Any]]:
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
            
            # Get available collections
            collections = await self.list_collections()
            if not collections:
                logger.warning("No collections available for search")
                return []
            
            collection_names = [col["name"] for col in collections]
            logger.info(f"Available collections: {len(collection_names)}")
            
            # Check if this is a vectorizer-related query
            query_lower = query.lower()
            is_vectorizer_query = any(term in query_lower for term in [
                'vectorizer', 'vector', 'embedding', 'search', 'rust', 'cargo'
            ])
            
            if is_vectorizer_query:
                # Prioritize vectorizer collections
                vectorizer_collections = [
                    col for col in collection_names 
                    if any(term in col.lower() for term in ['vectorizer', 'docs', 'source', 'sdk'])
                ]
                
                if vectorizer_collections:
                    logger.info(f"Vectorizer query detected, prioritizing: {vectorizer_collections}")
                    all_results = []
                    
                    for collection in vectorizer_collections[:3]:  # Limit to top 3
                        try:
                            results = await self.search_collection(collection, query, 3)
                            for item in results:
                                payload = item.get("payload", {})
                                processed_item = {
                                    "content": payload.get("content", ""),
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
                
                all_results = []
                for collection in prioritized_collections[:5]:  # Limit to top 5
                    try:
                        results = await self.search_collection(collection, query, 2)
                        for item in results:
                            payload = item.get("payload", {})
                            processed_item = {
                                "content": payload.get("content", ""),
                                "score": item.get("score", 0.0),
                                "collection": collection,
                                "doc_id": item.get("id", ""),
                                "metadata": payload if isinstance(payload, dict) else {}
                            }
                            all_results.append(processed_item)
                    except Exception as e:
                        logger.warning(f"Failed to search collection {collection}: {e}")
                        continue

            # Sort by score and limit results
            all_results.sort(key=lambda x: x["score"], reverse=True)
            final_results = all_results[:max_results]
            
            logger.info(f"Intelligent search completed: {len(final_results)} results")
            return final_results
                        
        except Exception as e:
            logger.error(f"Intelligent search failed: {e}")
            return []

    def _prioritize_collections(self, query: str, collections: List[str]) -> List[str]:
        """Prioriza coleções baseado na query"""
        query_lower = query.lower()
        
        # Keywords for prioritization
        priority_keywords = {
            'vectorizer': ['vectorizer', 'vector', 'embedding', 'search'],
            'docs': ['docs', 'documentation', 'guide', 'tutorial'],
            'source': ['source', 'code', 'implementation'],
            'sdk': ['sdk', 'client', 'api']
        }
        
        prioritized = []
        
        # Add collections that match query keywords
        for keyword, terms in priority_keywords.items():
            for term in terms:
                if term in query_lower:
                    matching_collections = [col for col in collections if keyword in col.lower()]
                    prioritized.extend(matching_collections)
        
        # Add remaining collections
        for col in collections:
            if col not in prioritized:
                prioritized.append(col)
        
        return prioritized

# =============================================================================
# BITNET SERVER
# =============================================================================

class BitNetServer:
    """Servidor BitNet com busca otimizada do Vectorizer e BitNet REAL"""
    
    def __init__(self):
        self.app = FastAPI(
            title="BitNet Server",
            description="Servidor BitNet com busca otimizada do Vectorizer e BitNet REAL",
            version="2.0"
        )
        
        # Setup CORS
        self.app.add_middleware(
            CORSMiddleware,
            allow_origins=["*"],
            allow_credentials=True,
            allow_methods=["*"],
            allow_headers=["*"],
        )
        
        # Initialize Vectorizer client
        self.vectorizer = VectorizerClient()
        
        # WebSocket connections
        self.active_connections: List[WebSocket] = []
        
        # Initialize BitNet
        self._init_bitnet()
        
        # Setup routes
        self._setup_routes()
    
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
        
        @self.app.get("/")
        async def root():
            """Root endpoint with API information"""
            return {
                "message": "BitNet FastAPI Server",
                "version": "2.0",
                "model": MODEL_NAME,
                "status": "running" if bitnet_loaded else "loading",
                "endpoints": {
                    "health": "/api/health",
                    "generate": "/generate",
                    "chat": "/api/chat",
                    "websocket": "/ws",
                    "info": "/info",
                    "docs": "/docs"
                }
            }
        
        @self.app.get("/chat", response_class=HTMLResponse)
        async def chat_page():
            """Página inicial com interface de chat"""
            return """
            <!DOCTYPE html>
            <html>
            <head>
                <title>BitNet Chat</title>
                <meta charset="utf-8">
                <style>
                    body { font-family: Arial, sans-serif; margin: 0; padding: 20px; background: #f5f5f5; }
                    .container { max-width: 800px; margin: 0 auto; background: white; border-radius: 10px; padding: 20px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }
                    .chat-container { height: 400px; border: 1px solid #ddd; padding: 10px; overflow-y: auto; margin-bottom: 20px; }
                    .message { margin: 10px 0; padding: 10px; border-radius: 5px; }
                    .user { background: #e3f2fd; text-align: right; }
                    .bot { background: #f1f8e9; }
                    .input-container { display: flex; gap: 10px; }
                    input[type="text"] { flex: 1; padding: 10px; border: 1px solid #ddd; border-radius: 5px; }
                    button { padding: 10px 20px; background: #2196f3; color: white; border: none; border-radius: 5px; cursor: pointer; }
                    button:hover { background: #1976d2; }
                    button:disabled { background: #ccc; cursor: not-allowed; }
                </style>
            </head>
            <body>
                <div class="container">
                    <h1>🤖 BitNet Chat</h1>
                    <div id="chat" class="chat-container"></div>
                    <div class="input-container">
                        <input type="text" id="messageInput" placeholder="Digite sua mensagem..." />
                        <button onclick="sendMessage()">Enviar</button>
                    </div>
                </div>
                
                <script>
                    const ws = new WebSocket('ws://localhost:15006/ws');
                    const chat = document.getElementById('chat');
                    const input = document.getElementById('messageInput');
                    
                    ws.onmessage = function(event) {
                        const data = JSON.parse(event.data);
                        if (data.type === 'response') {
                            addMessage(data.response, 'bot');
                        } else if (data.type === 'status') {
                            addMessage(data.message, 'status');
                        }
                    };
                    
                    function addMessage(text, type) {
                        const div = document.createElement('div');
                        div.className = `message ${type}`;
                        div.textContent = text;
                        chat.appendChild(div);
                        chat.scrollTop = chat.scrollHeight;
                    }
                    
                    function sendMessage() {
                        const message = input.value.trim();
                        if (message) {
                            addMessage(message, 'user');
                            ws.send(JSON.stringify({type: 'chat', message: message}));
                            input.value = '';
                        }
                    }
                    
                    input.addEventListener('keypress', function(e) {
                        if (e.key === 'Enter') {
                            sendMessage();
                        }
                    });
                </script>
            </body>
            </html>
            """
        
        @self.app.get("/api/health", response_model=HealthResponse)
        async def health():
            """Health check endpoint"""
            vectorizer_connected = await self.vectorizer.health_check()
            
            device_info = "unknown"
            gpu_info = {}
            
            if bitnet_loaded:
                device_info = str(next(bitnet_model.parameters()).device)
                
                if torch.cuda.is_available():
                    gpu_info = {
                        "gpu_available": True,
                        "gpu_count": torch.cuda.device_count(),
                        "gpu_name": torch.cuda.get_device_name(0),
                        "gpu_memory_total": torch.cuda.get_device_properties(0).total_memory,
                        "gpu_memory_allocated": torch.cuda.memory_allocated(0),
                        "gpu_memory_cached": torch.cuda.memory_reserved(0)
                    }
                else:
                    gpu_info = {"gpu_available": False}
            
            model_info = {
                "name": MODEL_NAME,
                "loaded": bitnet_loaded,
                "device": device_info,
                "gpu": gpu_info,
                "vectorizer_connected": vectorizer_connected,
                "timestamp": time.time()
            }
            
            return HealthResponse(
                status="healthy" if bitnet_loaded else "loading",
                model_loaded=bitnet_loaded,
                model_info=model_info
            )
        
        @self.app.post("/api/chat")
        async def chat(request: Dict[str, Any]):
            """Endpoint de chat principal"""
            try:
                message = request.get("message", "")
                history = request.get("history", [])
                
                if not message:
                    raise HTTPException(status_code=400, detail="Message is required")
                
                logger.info(f"Chat request: '{message}'")
                
                # Perform intelligent search
                search_results = await self.vectorizer.intelligent_search(message, max_results=5)
                logger.info(f"Search results: {len(search_results)}")
                
                # Build context
                context = self._build_context(search_results)
                logger.info(f"Context length: {len(context)} characters")
                
                # Generate response
                response = await self._generate_response(message, context, history)
                
                return {
                    "response": response,
                    "search_results": len(search_results),
                    "context_length": len(context),
                    "timestamp": time.time()
                }
                
            except Exception as e:
                logger.error(f"Chat error: {e}")
                raise HTTPException(status_code=500, detail=str(e))
        
        @self.app.post("/generate", response_model=GenerateResponse)
        async def generate(request: GenerateRequest):
            """Generate text using BitNet model"""
            
            if not bitnet_loaded:
                try:
                    await load_bitnet_model()
                except Exception as e:
                    raise HTTPException(status_code=503, detail=f"Model loading failed: {e}")
            
            try:
                start_time = time.time()
                
                # Debug: Log the entire request
                logger.info(f"🔍 Full request received: {request}")
                logger.info(f"🔍 Request context field: '{request.context}'")
                logger.info(f"🔍 Context type: {type(request.context)}")
                
                # Convert messages to simple format for context building
                message_text = ""
                for msg in request.messages:
                    if msg.role == "user":
                        message_text = msg.content
                        break
                
                # Use ONLY the context provided by server.js - NO additional search
                context = request.context or ""
                logger.info(f"✅ Using context from server.js: {len(context)} characters")
                if context:
                    logger.info(f"📝 Context preview: {context[:200]}...")
                
                # Generate response
                response = await self._generate_response(message_text, context, [])
                
                end_time = time.time()
                generation_time = end_time - start_time
                tokens_generated = len(response.split()) if response else 0
                
                model_info = {
                    "name": MODEL_NAME,
                    "tokens_generated": tokens_generated,
                    "generation_time": generation_time
                }
                
                return GenerateResponse(
                    response=response,
                    tokens_generated=tokens_generated,
                    generation_time=generation_time,
                    model_info=model_info
                )
                
            except Exception as e:
                logger.error(f"Generate endpoint error: {e}")
                raise HTTPException(status_code=500, detail=str(e))
        
        @self.app.websocket("/ws")
        async def websocket_endpoint(websocket: WebSocket):
            """WebSocket endpoint para chat em tempo real"""
            await websocket.accept()
            self.active_connections.append(websocket)
            
            try:
                while True:
                    data = await websocket.receive_text()
                    message_data = json.loads(data)
                    
                    if message_data.get("type") == "chat":
                        message = message_data.get("message", "")
                        
                        # Send status
                        await websocket.send_text(json.dumps({
                            "type": "status",
                            "message": "Searching knowledge base..."
                        }))
                        
                        # Perform search
                        search_results = await self.vectorizer.intelligent_search(message, max_results=5)
                        
                        await websocket.send_text(json.dumps({
                            "type": "status",
                            "message": f"Found {len(search_results)} results, generating response..."
                        }))
                        
                        # Build context and generate response
                        context = self._build_context(search_results)
                        response = await self._generate_response(message, context, [])
                        
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
        
        @self.app.get("/info")
        async def model_info():
            """Get detailed model information"""
            if not bitnet_loaded:
                raise HTTPException(status_code=503, detail="Model not loaded")
            
            return {
                "model_name": MODEL_NAME,
                "device": str(next(bitnet_model.parameters()).device),
                "dtype": str(next(bitnet_model.parameters()).dtype),
                "parameters": sum(p.numel() for p in bitnet_model.parameters()),
                "config": {
                    "max_tokens": MAX_TOKENS,
                    "temperature": TEMPERATURE,
                    "top_p": TOP_P
                }
            }
    
    def _build_context(self, search_results: List[Dict[str, Any]]) -> str:
        """Constrói contexto a partir dos resultados da busca"""
        if not search_results:
            return ""
        
        context_parts = []
        for result in search_results:
            collection = result.get("collection", "unknown")
            score = result.get("score", 0.0)
            content = result.get("content", "")
            
            context_parts.append(f"[{collection}] (score: {score:.3f})\n{content}")
        
        return "\n\n---\n\n".join(context_parts)
    
    async def _generate_response(self, message: str, context: str, history: List[Dict[str, Any]]) -> str:
        """Gera resposta usando BitNet REAL"""
        try:
            # Build prompt with better instructions
            prompt_parts = []
            
            if context:
                prompt_parts.append(f"""Based on the following information, answer the question:

{context}

Question: {message}

Answer:""")
            else:
                prompt_parts.append(f"""Answer this question:

Question: {message}

Answer:""")
            
            prompt = "\n\n".join(prompt_parts)
            
            logger.info(f"Context being sent to BitNet: {context[:200]}...")
            
            # Call BitNet model
            response = await self._call_bitnet_model(prompt)
            return response
        
        except Exception as e:
            logger.error(f"Response generation failed: {e}")
            return f"Erro ao gerar resposta: {e}"
    
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

if __name__ == "__main__":
    import uvicorn
    
    # Create server instance
    server = BitNetServer()
    
    # Start server
    logger.info("🚀 Starting BitNet Server...")
    uvicorn.run(
        server.app,
        host="0.0.0.0",
        port=15006,
        log_level="info"
    )

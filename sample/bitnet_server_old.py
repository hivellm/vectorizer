#!/usr/bin/env python3
"""
BitNet FastAPI Server
Serves the BitNet b1.58 2B4T model via REST API
"""

import asyncio
import json
import os
import sys
import logging
from pathlib import Path
from typing import List, Dict, Optional, Any
import uvicorn
from fastapi import FastAPI, HTTPException
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel
import torch
from transformers import AutoTokenizer, AutoModelForCausalLM
import time
import httpx
import json

# Disable PyTorch compilation to avoid C++ compiler issues
os.environ['TORCH_COMPILE_DISABLED'] = '1'
os.environ['TORCHDYNAMO_DISABLE'] = '1'

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)

# Model configuration
MODEL_NAME = "microsoft/bitnet-b1.58-2B-4T"
MODEL_PATH = Path(__file__).parent / "models" / "BitNet-b1.58-2B-4T" / "ggml-model-i2_s.gguf"
MAX_TOKENS = 512
TEMPERATURE = 0.7
TOP_P = 0.9

# Global variables
model = None
tokenizer = None
model_loaded = False

# MCP Client configuration - Updated to use Vectorizer's unified server
MCP_SERVER_URL = "http://localhost:15002"

class MCPClient:
    """Client for communicating with the MCP Channel"""
    
    def __init__(self, base_url: str = MCP_SERVER_URL):
        self.base_url = base_url
        self.client = httpx.AsyncClient(timeout=30.0)
    
    async def call_tool(self, tool: str, args: Dict[str, Any] = None) -> Dict[str, Any]:
        """Call an MCP tool"""
        try:
            # Use the correct MCP endpoint
            mcp_url = f"{self.base_url}/mcp"
            response = await self.client.post(
                mcp_url,
                json={"tool": tool, "args": args or {}}
            )
            response.raise_for_status()
            return response.json()
        except Exception as e:
            logger.warning(f"MCP call failed: {e}")
            return {"error": str(e)}
    
    async def list_collections(self) -> List[Dict[str, Any]]:
        """List all collections using REST API"""
        try:
            response = await self.client.get(f"{self.base_url}/collections")
            response.raise_for_status()
            result = response.json()
            logger.info(f"📚 Available collections: {result}")
            
            # Extract collections from the API response
            if isinstance(result, dict) and "collections" in result:
                collections = result["collections"]
                # Filter collections with data
                collections_with_data = [col for col in collections if col.get("document_count", 0) > 0]
                logger.info(f"📊 Found {len(collections_with_data)} collections with data out of {len(collections)} total")
                return collections_with_data
            elif isinstance(result, list):
                return result
            else:
                logger.warning(f"Unexpected collections response format: {type(result)}")
                return []
        except Exception as e:
            logger.warning(f"Failed to list collections: {e}")
            return []
    
    async def _prioritize_collections(self, query: str, collections: List[str]) -> List[str]:
        """Intelligently prioritize collections using semantic similarity between query and collection names"""
        query_lower = query.lower()
        
        # Extract key terms from query
        query_terms = [term.strip() for term in query_lower.split() if len(term.strip()) > 2]
        
        # If query is too short or generic, return all collections
        if len(query_terms) < 1:
            return collections
        
        # Use semantic search to find most relevant collections
        try:
            # Create a semantic search query that includes collection names
            semantic_query = f"{query} collections: {', '.join(collections[:20])}"
            
            # Use intelligent search to find collections most relevant to the query
            response = await self.client.post(
                f"{self.base_url}/intelligent_search",
                json={
                    "query": semantic_query,
                    "collections": collections[:20],  # Test first 20 collections
                    "max_results": 10,
                    "mmr_enabled": True,  # Enable MMR for better diversity
                    "domain_expansion": True,  # Enable domain expansion for better coverage
                    "technical_focus": True,
                    "mmr_lambda": 0.7
                }
            )
            
            if response.status_code == 200:
                result = response.json()
                results = result.get("results", [])
                
                # Extract collection names from results and their scores
                collection_scores = {}
                for item in results:
                    collection_name = item.get("collection", "")
                    score = item.get("score", 0.0)
                    if collection_name and collection_name not in collection_scores:
                        collection_scores[collection_name] = score
                
                # Sort by semantic relevance score
                sorted_collections = sorted(
                    collection_scores.items(),
                    key=lambda x: x[1],
                    reverse=True
                )
                
                # Get top collections
                prioritized = [name for name, score in sorted_collections if score > -0.5]
                
                # Add remaining collections that weren't found in semantic search
                remaining = [col for col in collections if col not in collection_scores]
                
                result = prioritized + remaining
                
                logger.info(f"📊 Semantic prioritization: {len(prioritized)} relevant, {len(remaining)} remaining")
                if prioritized:
                    logger.info(f"🎯 Top semantic matches: {prioritized[:3]}")
                
                return result
                
        except Exception as e:
            logger.warning(f"Semantic prioritization failed: {e}")
        
        # Fallback: Use name-based pattern matching
        logger.info("🔄 Falling back to pattern-based prioritization")
        return self._fallback_pattern_prioritization(query, collections)
    
    def _fallback_pattern_prioritization(self, query: str, collections: List[str]) -> List[str]:
        """Fallback pattern-based prioritization when semantic search fails"""
        query_lower = query.lower()
        
        # Extract meaningful terms from query
        query_terms = [term.strip() for term in query_lower.split() if len(term.strip()) > 2]
        
        # Score collections based on term overlap
        collection_scores = {}
        for collection in collections:
            collection_lower = collection.lower()
            score = 0.0
            
            # Direct term matches get highest score
            for term in query_terms:
                if term in collection_lower:
                    score += 1.0
            
            # Partial matches get lower score
            for term in query_terms:
                if any(term in part for part in collection_lower.split('-')):
                    score += 0.5
            
            collection_scores[collection] = score
        
        # Sort by score
        sorted_collections = sorted(
            collection_scores.items(),
            key=lambda x: x[1],
            reverse=True
        )
        
        result = [name for name, score in sorted_collections]
        logger.info(f"📊 Pattern prioritization: {len([s for s in collection_scores.values() if s > 0])} matches")
        
        return result
    
    async def intelligent_search(self, query: str, max_results: int = 5) -> List[Dict[str, Any]]:
        """Perform intelligent search using optimized approach for speed"""
        try:
            # Skip Step 1 (intelligent_search) and go directly to Step 2 (fallback)
            # because intelligent_search is returning irrelevant results
            logger.info(f"🔍 Using direct collection search for: '{query}'")
            
            # Step 2: Direct search in relevant collections
            logger.info(f"🔍 Searching relevant collections directly")
            
            # Get collections and prioritize by name
            collections = await self.list_collections()
            collection_names = [col["name"] for col in collections if col.get("document_count", 0) > 0]
            
            # Prioritize collections that might contain vectorizer content
            prioritized_collections = []
            query_lower = query.lower()
            
            # Look for vectorizer-related collections first
            for col in collection_names:
                if 'vectorizer' in col.lower():
                    prioritized_collections.append(col)
            
            # Add other potentially relevant collections
            for col in collection_names:
                if col not in prioritized_collections and ('docs' in col.lower() or 'source' in col.lower()):
                    prioritized_collections.append(col)
            
            # Add remaining collections
            for col in collection_names:
                if col not in prioritized_collections:
                    prioritized_collections.append(col)
            
            logger.info(f"🎯 Searching in {len(prioritized_collections[:5])} prioritized collections: {prioritized_collections[:5]}")
            
            # Search top collections
            all_results = []
            for collection in prioritized_collections[:5]:  # Limit to top 5
                try:
                    collection_response = await self.client.post(
                        f"{self.base_url}/collections/{collection}/search/text",
                        json={
                            "query": query,
                            "limit": 2  # Get top 2 from each collection
                        },
                        timeout=5.0
                    )
                    
                    if collection_response.status_code == 200:
                        collection_result = collection_response.json()
                        if "results" in collection_result:
                            for item in collection_result["results"]:
                                if isinstance(item, dict):
                                    # Extract content from payload structure
                                    payload = item.get("payload", {})
                                    content = payload.get("content", "") if isinstance(payload, dict) else ""
                                    
                                    # Handle encoding issues
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
                    logger.warning(f"Failed to search collection {collection}: {e}")
                    continue
            
            # Sort by score and return top results
            all_results.sort(key=lambda x: x.get("score", 0.0), reverse=True)
            final_results = all_results[:max_results]
            
            logger.info(f"✅ Found {len(final_results)} results from direct collection search")
            return final_results
                
        except Exception as e:
            logger.warning(f"All search methods failed: {e}")
            return []
    
    async def _fallback_simple_search(self, query: str, collections: List[str], max_results: int) -> List[Dict[str, Any]]:
        """Fallback to simple search when intelligent search fails"""
        processed_results = []
        
        for collection in collections:
            try:
                response = await self.client.post(
                    f"{self.base_url}/collections/{collection}/search/text",
                    json={
                        "query": query,
                        "limit": max_results // len(collections) + 1  # Distribute results across collections
                    }
                )
                response.raise_for_status()
                result = response.json()
                
                if "results" in result:
                    for item in result["results"]:
                        if isinstance(item, dict):
                            # Extract text from payload
                            payload = item.get("payload", {})
                            content = payload.get("text", "")
                            
                            processed_item = {
                                "content": content,
                                "score": item.get("score", 0.0),
                                "metadata": payload,
                                "collection": collection
                            }
                            processed_results.append(processed_item)
                            
            except Exception as e:
                logger.warning(f"Simple search failed for collection {collection}: {e}")
                continue
        
        logger.info(f"🔄 Fallback simple search returned {len(processed_results)} results from {len(collections)} collections")
        return processed_results
    
    async def get_collection_info(self, collection: str) -> Dict[str, Any]:
        """Get collection information"""
        result = await self.call_tool("get_collection_info", {"collection": collection})
        if "result" in result and "content" in result["result"]:
            try:
                info = json.loads(result["result"]["content"][0]["text"])
                return info
            except:
                return {}
        return {}
    
    async def close(self):
        """Close the HTTP client"""
        await self.client.aclose()

# Global MCP client
mcp_client = None

# FastAPI app
app = FastAPI(
    title="BitNet API Server",
    description="REST API server for BitNet b1.58 2B4T model",
    version="1.0.0"
)

# CORS middleware
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

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

class HealthResponse(BaseModel):
    status: str
    model_loaded: bool
    model_info: Dict[str, Any]

def check_model_file():
    """Check if the BitNet model file exists"""
    if not MODEL_PATH.exists():
        raise FileNotFoundError(f"BitNet model not found at: {MODEL_PATH}")
    logger.info(f"✅ BitNet model found: {MODEL_PATH}")

async def load_model():
    """Load the BitNet model and tokenizer with GPU optimization"""
    global model, tokenizer, model_loaded
    
    try:
        logger.info("🔄 Loading BitNet model and tokenizer...")
        
        # Check GPU availability with better detection
        gpu_available = False
        gpu_info = {}
        
        try:
            if torch.cuda.is_available():
                gpu_count = torch.cuda.device_count()
                gpu_name = torch.cuda.get_device_name(0)
                logger.info(f"🚀 GPU detected: {gpu_name} (Count: {gpu_count})")
                
                # Test GPU memory
                torch.cuda.empty_cache()
                test_tensor = torch.randn(100, 100).cuda()
                del test_tensor
                torch.cuda.empty_cache()
                
                gpu_available = True
                gpu_info = {
                    "name": gpu_name,
                    "count": gpu_count,
                    "memory_total": torch.cuda.get_device_properties(0).total_memory
                }
            else:
                logger.info("💻 No CUDA GPU detected, checking for other GPU types...")
                
                # Check for other GPU types (ROCm, DirectML, etc.)
                if hasattr(torch.backends, 'mps') and torch.backends.mps.is_available():
                    logger.info("🍎 Apple Metal Performance Shaders (MPS) detected")
                    gpu_available = True
                    gpu_info = {"name": "Apple MPS", "type": "mps"}
                elif hasattr(torch.backends, 'directml') and torch.backends.directml.is_available():
                    logger.info("🪟 DirectML GPU detected")
                    gpu_available = True
                    gpu_info = {"name": "DirectML", "type": "directml"}
                else:
                    logger.info("🖥️ No GPU acceleration available, using CPU")
                    
        except Exception as e:
            logger.warning(f"⚠️ GPU detection failed: {e}, falling back to CPU")
            gpu_available = False
        
        # Load tokenizer
        tokenizer = AutoTokenizer.from_pretrained(MODEL_NAME)
        logger.info("✅ Tokenizer loaded")
        
        # Load model based on GPU availability
        if gpu_available and torch.cuda.is_available():
            try:
                # Load model with GPU optimization
                model = AutoModelForCausalLM.from_pretrained(
                    MODEL_NAME,
                    torch_dtype=torch.bfloat16,
                    device_map="auto",
                    low_cpu_mem_usage=True,
                    trust_remote_code=True
                )
                
                # Move model to GPU and optimize
                model = model.cuda()
                model.eval()
                
                # Enable memory efficient attention
                if hasattr(model, 'gradient_checkpointing_enable'):
                    model.gradient_checkpointing_enable()
                
                logger.info("✅ Model loaded on GPU with optimizations")
                
            except Exception as e:
                logger.warning(f"⚠️ GPU loading failed: {e}, falling back to CPU")
                gpu_available = False
        
        if not gpu_available:
            # Load model for CPU
            model = AutoModelForCausalLM.from_pretrained(
                MODEL_NAME,
                torch_dtype=torch.float32,
                device_map="cpu",
                low_cpu_mem_usage=True
            )
            logger.info("✅ Model loaded on CPU")
        
        model_loaded = True
        logger.info("🎉 BitNet model ready for inference!")
        
    except Exception as e:
        logger.error(f"❌ Failed to load model: {e}")
        raise

def format_chat_messages(messages: List[ChatMessage], context: str = "") -> str:
    """Format chat messages into a prompt for the model"""
    formatted_messages = []
    
    # Add system message with context if provided
    if context:
        logger.info(f"📝 Context being sent to BitNet: '{context[:500]}...' (total: {len(context)} chars)")
        formatted_messages.append({
            "role": "system",
            "content": f"""You are BitNet, a helpful AI assistant. Answer the user's question using ONLY the information provided in the context below.

RULES:
- Use ONLY information from the context
- Be concise and specific (under 200 words)
- If context is insufficient, say "I need more information about [topic]"
- Do NOT invent or make up information

CONTEXT:
{context}

Answer the question directly and clearly based on the context above."""
        })
    else:
        formatted_messages.append({
            "role": "system", 
            "content": "You are BitNet, a helpful AI assistant. Provide concise, specific answers under 200 words."
        })
    
    # Add conversation messages
    for msg in messages:
        formatted_messages.append({
            "role": msg.role,
            "content": msg.content
        })
    
    # Apply chat template
    try:
        prompt = tokenizer.apply_chat_template(
            formatted_messages,
            tokenize=False,
            add_generation_prompt=True
        )
    except Exception as e:
        logger.warning(f"Failed to apply chat template: {e}")
        # Fallback: simple format
        prompt = "\n".join([f"{msg['role']}: {msg['content']}" for msg in formatted_messages])
        prompt += "\nassistant:"
    
    return prompt

async def perform_intelligent_search(query: str) -> str:
    """Perform intelligent search using optimized Vectorizer configuration"""
    global mcp_client
    
    if not mcp_client:
        logger.warning("⚠️ MCP client not available")
        return ""  # Return empty instead of old context
    
    try:
        logger.info(f"🔍 Performing intelligent search for: '{query}'")
        
        # First check available collections
        collections = await mcp_client.list_collections()
        if not collections:
            logger.warning("⚠️ No collections available for search")
            return ""
        
        # Try a simple search first to test if there's any data
        try:
            simple_search_response = await mcp_client.client.post(
                f"{mcp_client.base_url}/search",
                json={
                    "query": query,
                    "limit": 3
                }
            )
            simple_result = simple_search_response.json()
            logger.info(f"🔍 Simple search test: {json.dumps(simple_result, indent=2)}")
        except Exception as e:
            logger.warning(f"Simple search test failed: {e}")
        
        # Use the optimized intelligent search
        search_results = await mcp_client.intelligent_search(query, max_results=5)
        
        if search_results:
            logger.info(f"✅ Found {len(search_results)} relevant results")
            context_parts = []
            
            for result in search_results:
                content = result.get('content', '')
                score = result.get('score', 0.0)
                collection = result.get('collection', 'unknown')
                
                # Log the score for debugging
                logger.info(f"📊 Result from {collection}: score={score:.3f}, content_length={len(content)}")
                
                # Lower threshold to include more results (BM25 scores can be negative)
                if score >= -0.1:  # Much lower threshold for BM25 scores
                    # Truncate content to avoid overwhelming the model
                    if len(content) > 500:
                        content = content[:500] + "..."
                    
                    formatted_content = f"[{collection}] (score: {score:.3f})\n{content}"
                    context_parts.append(formatted_content)
            
            if context_parts:
                enhanced_context = "\n\n".join(context_parts)
                logger.info(f"📝 Context created: {len(enhanced_context)} chars from {len(context_parts)} sources")
                return enhanced_context
            else:
                logger.warning("⚠️ No high-quality results found")
                return ""  # Return empty instead of old context
        else:
            logger.warning("⚠️ No results found for context enhancement")
            return ""  # Return empty instead of old context
            
    except Exception as e:
        logger.warning(f"Intelligent search failed: {e}")
        return ""  # Return empty instead of old context

# Removed helper functions - now handled by Vectorizer's optimized intelligent_search

def extract_context_request(response: str) -> Dict[str, Any]:
    """Extract context request from BitNet response"""
    import json
    import re
    
    try:
        # Look for JSON pattern in response
        json_pattern = r'\{[^{}]*"request_context"[^{}]*\}'
        matches = re.findall(json_pattern, response)
        
        for match in matches:
            try:
                parsed = json.loads(match)
                if "request_context" in parsed:
                    return parsed["request_context"]
            except json.JSONDecodeError:
                continue
        
        # Also try to find JSON at the end of response
        lines = response.strip().split('\n')
        for line in reversed(lines):
            if line.strip().startswith('{') and '"request_context"' in line:
                try:
                    parsed = json.loads(line.strip())
                    if "request_context" in parsed:
                        return parsed["request_context"]
                except json.JSONDecodeError:
                    continue
                    
    except Exception:
        pass
    
    return None

# Simplified context handling - now uses intelligent_search directly
async def get_additional_context(query: str, collections: List[str] = None) -> str:
    """Get additional context using optimized intelligent search"""
    global mcp_client
    
    if not mcp_client:
        return ""
    
    try:
        # Use intelligent search for better results
        search_results = await mcp_client.intelligent_search(query, max_results=3)
        
        if search_results:
            context_parts = []
            for result in search_results:
                content = result.get('content', '')
                score = result.get('score', 0.0)
                collection = result.get('collection', 'unknown')
                
                if score >= -0.1:  # Lower threshold for BM25 scores
                    context_parts.append(f"[{collection} (score: {score:.3f})] {content[:300]}")
            
            return "\n".join(context_parts)
        
        return ""
        
    except Exception as e:
        logger.warning(f"Failed to get additional context: {e}")
        return ""

async def generate_response_with_iterative_mcp(
    messages: List[ChatMessage], 
    context: str = "", 
    use_mcp_context: bool = True
) -> tuple[str, int, float]:
    """Generate response using BitNet model with iterative MCP context requests"""
    
    logger.info(f"🔄 Starting iterative MCP generation with {len(messages)} messages")
    
    if not model_loaded:
        raise HTTPException(status_code=503, detail="Model not loaded")
    
    start_time = time.time()
    max_iterations = 2  # Reduced iterations
    current_context = context
    iteration = 0
    max_generation_time = 300  # 5 minutes max per generation
    
    while iteration < max_iterations:
        iteration += 1
        logger.info(f"🔄 MCP Iteration {iteration}/{max_iterations}")
        
        try:
            # Enhance context with MCP if enabled
            enhanced_context = current_context
            if use_mcp_context and mcp_client and messages:
                logger.info(f"🔧 MCP client available: {mcp_client is not None}")
                try:
                    # Get the last user message for context enhancement
                    last_user_message = None
                    for msg in reversed(messages):
                        if msg.role == "user":
                            last_user_message = msg.content
                            break
                    
                    if last_user_message:
                        logger.info(f"🔍 Enhancing context for query: '{last_user_message}'")
                        enhanced_context = await perform_intelligent_search(last_user_message)
                        logger.info(f"📝 Enhanced context length: {len(enhanced_context)} chars")
                        logger.info(f"📝 Context preview: {enhanced_context[:200]}...")
                        
                except Exception as e:
                    logger.warning(f"MCP context enhancement failed: {e}")
            else:
                logger.warning(f"⚠️ MCP context disabled - use_mcp_context: {use_mcp_context}, mcp_client: {mcp_client is not None}, messages: {len(messages) if messages else 0}")
            
            # Format the prompt
            prompt = format_chat_messages(messages, enhanced_context)
            
            # Tokenize input
            inputs = tokenizer(prompt, return_tensors="pt")
            
            # Move to same device as model
            device = next(model.parameters()).device
            inputs = {k: v.to(device) for k, v in inputs.items()}
            
            # Generate response with technical parameters optimized for JSON
            generation_kwargs = {
                "max_new_tokens": 150,  # Increased to ensure complete JSON
                "temperature": 0.05,    # Very low temperature for consistent JSON
                "top_p": 0.6,          # Very focused generation
                "do_sample": True,
                "pad_token_id": tokenizer.eos_token_id,
                "eos_token_id": tokenizer.eos_token_id,
                "repetition_penalty": 1.2,
                "no_repeat_ngram_size": 2,
                "min_length": 10,      # Ensure minimum length
                "use_cache": True,
                "early_stopping": False  # Disable early stopping to ensure complete JSON
            }
            
            
            # Check if we're running out of time
            if time.time() - start_time > max_generation_time:
                logger.warning("⏰ Generation timeout, using fallback")
                break
            
            with torch.no_grad():
                outputs = model.generate(**inputs, **generation_kwargs)
            
            # Decode response
            input_length = inputs['input_ids'].shape[1]
            response_tokens = outputs[0][input_length:]
            response = tokenizer.decode(response_tokens, skip_special_tokens=True)
            
            logger.info(f"📝 Raw response: '{response[:200]}...' (length: {len(response)})")

            # Check if we have a valid response
            response_clean = response.strip()
            if not response_clean or len(response_clean) <= 10:
                logger.warning("⚠️ Empty or too short response, trying next iteration")
                continue

            # Return the response directly (no JSON parsing)
            logger.info(f"✅ Generated response: '{response_clean[:100]}...'")
            end_time = time.time()
            generation_time = end_time - start_time
            tokens_generated = len(response_clean.split())
            return response_clean, tokens_generated, generation_time
            
        except Exception as e:
            logger.error(f"❌ Generation failed on iteration {iteration}: {e}")
            if iteration >= max_iterations:
                break
    
    # If we get here, all iterations failed
    logger.error("🚨 All iterations failed - NO FALLBACK")
    raise HTTPException(status_code=500, detail="BitNet failed to generate a proper response after multiple attempts")

async def generate_response(
    messages: List[ChatMessage],
    context: str = "",
    max_tokens: int = MAX_TOKENS,
    temperature: float = TEMPERATURE,
    top_p: float = TOP_P,
    use_mcp_context: bool = True
) -> tuple[str, int, float]:
    """Generate response using BitNet model"""
    
    if not model_loaded:
        raise HTTPException(status_code=503, detail="Model not loaded")
    
    start_time = time.time()
    
    try:
        # Enhance context with MCP if enabled
        enhanced_context = context
        if use_mcp_context and mcp_client and messages:
            try:
                # Get the last user message for context enhancement
                last_user_message = None
                for msg in reversed(messages):
                    if msg.role == "user":
                        last_user_message = msg.content
                        break
                
                if last_user_message:
                    # Intelligent iterative search like Cursor does
                    enhanced_context = await perform_intelligent_search(last_user_message)
                        
            except Exception as e:
                logger.warning(f"MCP context enhancement failed: {e}")
                # Continue with original context
        
        # Format the prompt with MCP instructions
        prompt = format_chat_messages(messages, enhanced_context)
        
        # Simplified prompt without complex instructions
        
        # Add instruction for dynamic context requests
        if use_mcp_context and mcp_client:
            prompt += "\n\nIMPORTANT: If you need more specific information to answer accurately, you can request additional context by responding with a JSON object like: {\"request_context\": {\"query\": \"specific question\", \"collections\": [\"collection_name\"]}}. This will trigger additional searches to provide you with more relevant information."
        
        # Tokenize input
        inputs = tokenizer(prompt, return_tensors="pt")
        
        # Move to same device as model
        device = next(model.parameters()).device
        inputs = {k: v.to(device) for k, v in inputs.items()}
        
        # Generate response with multiple fallback strategies
        response = ""
        generation_attempts = [
            # Attempt 1: Technical and specific parameters
            {
                "max_new_tokens": 200,
                "temperature": 0.3,
                "top_p": 0.9,
                "do_sample": True,
                "pad_token_id": tokenizer.eos_token_id,
                "eos_token_id": tokenizer.eos_token_id,
                "repetition_penalty": 1.1,
                "no_repeat_ngram_size": 2,
                "min_length": 20,
                "use_cache": True
            },
            # Attempt 2: More focused generation
            {
                "max_new_tokens": 150,
                "temperature": 0.5,
                "top_p": 0.95,
                "do_sample": True,
                "pad_token_id": tokenizer.eos_token_id,
                "eos_token_id": tokenizer.eos_token_id,
                "repetition_penalty": 1.05,
                "no_repeat_ngram_size": 1,
                "min_length": 15,
                "use_cache": True
            },
            # Attempt 3: Conservative fallback
            {
                "max_new_tokens": 100,
                "temperature": 0.7,
                "top_p": 1.0,
                "do_sample": True,
                "pad_token_id": tokenizer.eos_token_id,
                "eos_token_id": tokenizer.eos_token_id,
                "repetition_penalty": 1.0,
                "use_cache": False
            }
        ]
        
        for i, generation_kwargs in enumerate(generation_attempts):
            try:
                
                with torch.no_grad():
                    outputs = model.generate(**inputs, **generation_kwargs)
                
                # Decode response
                input_length = inputs['input_ids'].shape[1]
                response_tokens = outputs[0][input_length:]
                response = tokenizer.decode(response_tokens, skip_special_tokens=True)
                
                logger.info(f"📝 Attempt {i+1} response: '{response[:100]}...' (length: {len(response)})")
                
                # Check if response is valid
                if response and len(response.strip()) >= 3:
                    logger.info(f"✅ Successful generation on attempt {i+1}")
                    break
                else:
                    logger.warning(f"⚠️ Attempt {i+1} failed: empty or too short response")
                    
            except Exception as e:
                logger.warning(f"⚠️ Attempt {i+1} failed with error: {e}")
                continue
        
        # If all attempts failed, provide a technical fallback
        if not response or len(response.strip()) < 3:
            logger.warning("🚨 All generation attempts failed, using technical fallback")
            
            # Extract specific technical information from context
            technical_details = []
            if enhanced_context and len(enhanced_context) > 100:
                lines = enhanced_context.split('\n')
                
                # Extract URLs, endpoints, configurations
                for line in lines:
                    if 'http' in line.lower() or 'localhost' in line.lower():
                        technical_details.append(f"Endpoint: {line.strip()}")
                    elif 'config' in line.lower() or 'json' in line.lower():
                        technical_details.append(f"Configuration: {line.strip()}")
                    elif 'collection' in line.lower() and 'score' in line.lower():
                        technical_details.append(f"Collection data: {line.strip()}")
                    elif any(term in line.lower() for term in ['api', 'mcp', 'vectorizer', 'search']):
                        technical_details.append(f"Technical info: {line.strip()}")
                
                # Extract collection names
                collections = set()
                for line in lines:
                    if '[' in line and ']' in line:
                        try:
                            collection = line.split('[')[1].split(' ')[0]
                            collections.add(collection)
                        except:
                            pass
                
                if collections:
                    collection_list = ', '.join(list(collections)[:3])
                    technical_details.append(f"Available collections: {collection_list}")
            
            # Create technical response
            if technical_details:
                response = f"Based on Vectorizer MCP data: {'; '.join(technical_details[:3])}. For specific implementation details, please ask about particular features or configurations."
            else:
                response = "I need more specific information to provide a helpful answer. Please ask about particular features or configurations."
        
        logger.info(f"Final response: '{response[:100]}...' (length: {len(response)})")
        
        end_time = time.time()
        generation_time = end_time - start_time
        tokens_generated = len(response.split()) if response else 0
        
        return response.strip(), tokens_generated, generation_time
        
        # Check for context request and handle iteratively
        if use_mcp_context and mcp_client:
            context_request = extract_context_request(response)
            if context_request:
                logger.info(f"🔄 BitNet requested additional context: {context_request}")
                additional_context = await handle_context_request(context_request)
                if additional_context:
                    # Create enhanced prompt with additional context
                    enhanced_prompt = f"{prompt}\n\nAdditional Context: {additional_context}\n\nNow provide a comprehensive answer using all available context:"
                    
                    # Tokenize enhanced prompt
                    enhanced_inputs = tokenizer(enhanced_prompt, return_tensors="pt")
                    enhanced_inputs = {k: v.to(device) for k, v in enhanced_inputs.items()}
                    
                    # Generate enhanced response with adjusted parameters
                    with torch.no_grad():
                        enhanced_outputs = model.generate(
                            **enhanced_inputs,
                            max_new_tokens=min(max_tokens, 300),  # Increased limit
                            temperature=0.7,  # Slightly lower temperature
                            top_p=0.9,  # Slightly lower top_p
                            do_sample=True,
                            pad_token_id=tokenizer.eos_token_id,
                            eos_token_id=tokenizer.eos_token_id,
                            repetition_penalty=1.05,  # Reduced penalty
                            no_repeat_ngram_size=3
                        )
                    
                    # Decode enhanced response
                    enhanced_input_length = enhanced_inputs['input_ids'].shape[1]
                    enhanced_response_tokens = enhanced_outputs[0][enhanced_input_length:]
                    enhanced_response = tokenizer.decode(enhanced_response_tokens, skip_special_tokens=True)
                    
                    generation_time = time.time() - start_time
                    tokens_generated = len(enhanced_response_tokens)
                    
                    # Debug response
                    logger.info(f"Raw enhanced response: '{enhanced_response}'")
                    logger.info(f"Enhanced response length: {len(enhanced_response)}")
                    
                    # Ensure we have a valid response
                    if not enhanced_response or enhanced_response.strip() == "" or len(enhanced_response.strip()) < 5:
                        logger.warning("Empty or too short enhanced response detected")
                        enhanced_response = "I apologize, but I'm having trouble generating a response. Please try rephrasing your question."
                    
                    logger.info(f"Generated {tokens_generated} tokens in {generation_time:.2f}s")
                    
                    return enhanced_response.strip(), tokens_generated, generation_time
        
        # Check if BitNet requested more context
        context_request = extract_context_request(response)
        
        if context_request and use_mcp_context and mcp_client:
            logger.info(f"BitNet requested more context: {context_request}")
            
            # Get additional context
            additional_context = await handle_context_request(context_request)
            
            if additional_context:
                # Generate a new response with enhanced context
                enhanced_prompt = format_chat_messages(messages, enhanced_context + "\n\nAdditional Context: " + additional_context)
                
                # Tokenize enhanced prompt
                enhanced_inputs = tokenizer(enhanced_prompt, return_tensors="pt")
                enhanced_inputs = {k: v.to(device) for k, v in enhanced_inputs.items()}
                
                # Generate enhanced response with adjusted parameters
                with torch.no_grad():
                    enhanced_outputs = model.generate(
                        **enhanced_inputs,
                        max_new_tokens=min(max_tokens, 300),  # Increased limit
                        temperature=0.7,  # Slightly lower temperature
                        top_p=0.9,  # Slightly lower top_p
                        do_sample=True,
                        pad_token_id=tokenizer.eos_token_id,
                        eos_token_id=tokenizer.eos_token_id,
                        repetition_penalty=1.05,  # Reduced penalty
                        no_repeat_ngram_size=3
                    )
                
                # Decode enhanced response
                enhanced_input_length = enhanced_inputs['input_ids'].shape[1]
                enhanced_response_tokens = enhanced_outputs[0][enhanced_input_length:]
                enhanced_response = tokenizer.decode(enhanced_response_tokens, skip_special_tokens=True)
                
                generation_time = time.time() - start_time
                tokens_generated = len(enhanced_response_tokens)
                
                logger.info(f"Generated enhanced response with {tokens_generated} tokens in {generation_time:.2f}s")
                return enhanced_response.strip(), tokens_generated, generation_time
        
        generation_time = time.time() - start_time
        tokens_generated = len(response_tokens)
        
        # Debug response
        logger.info(f"Raw response: '{response}'")
        logger.info(f"Response length: {len(response)}")
        
        # Ensure we have a valid response
        if not response or response.strip() == "" or len(response.strip()) < 3:
            logger.warning("Empty or too short response detected")
            
            # Try alternative generation with different parameters
            logger.info("🔄 Retrying with alternative parameters...")
            with torch.no_grad():
                alt_outputs = model.generate(
                    **inputs,
                    max_new_tokens=50,
                    temperature=1.0,
                    top_p=1.0,
                    do_sample=True,
                    pad_token_id=tokenizer.eos_token_id,
                    eos_token_id=tokenizer.eos_token_id,
                    repetition_penalty=1.0,
                    no_repeat_ngram_size=0,
                    min_length=1,
                    use_cache=True
                )
                
                alt_response_tokens = alt_outputs[0][input_length:]
                alt_response = tokenizer.decode(alt_response_tokens, skip_special_tokens=True)
                
                if alt_response and len(alt_response.strip()) > 3:
                    response = alt_response
                    logger.info(f"✅ Alternative generation successful: '{response[:50]}...'")
                else:
                    # Provide a contextual fallback based on available information
                    if context and len(context) > 100:
                        collections = set()
                        for line in context.split('\n'):
                            if '[' in line and ']' in line:
                                try:
                                    collection = line.split('[')[1].split(' ')[0]
                                    collections.add(collection)
                                except:
                                    pass
                        if collections:
                            response = f"Based on the available information in collections like {', '.join(list(collections)[:3])}, I can help you with questions about these topics. Please ask a more specific question."
                        else:
                            response = "I can help you with questions about the available knowledge base. Please ask a specific question."
                    else:
                        response = "I'm here to help! Please ask me a specific question."
        
        logger.info(f"Generated {tokens_generated} tokens in {generation_time:.2f}s")
        
        return response.strip(), tokens_generated, generation_time
        
    except Exception as e:
        logger.error(f"Generation error: {e}")
        # NO FALLBACK - fail properly
        raise HTTPException(status_code=500, detail=f"Generation failed: {str(e)}")

@app.on_event("startup")
async def startup_event():
    """Initialize the model and MCP client on startup"""
    global mcp_client
    
    try:
        check_model_file()
        await load_model()
        
        # Initialize MCP client
        mcp_client = MCPClient()
        logger.info("🔧 MCP Client initialized")
        
        # Test MCP connection
        try:
            collections = await mcp_client.list_collections()
            logger.info(f"✅ MCP test successful - found {len(collections)} collections")
        except Exception as e:
            logger.error(f"❌ MCP test failed: {e}")
            mcp_client = None
        
    except Exception as e:
        logger.error(f"Startup failed: {e}")
        sys.exit(1)

@app.get("/health", response_model=HealthResponse)
async def health_check():
    """Health check endpoint with GPU information"""
    device_info = "unknown"
    gpu_info = {}
    
    if model_loaded:
        device_info = str(next(model.parameters()).device)
        
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
        "path": str(MODEL_PATH),
        "loaded": model_loaded,
        "device": device_info,
        "gpu": gpu_info
    }
    
    return HealthResponse(
        status="healthy" if model_loaded else "loading",
        model_loaded=model_loaded,
        model_info=model_info
    )

@app.post("/generate", response_model=GenerateResponse)
async def generate(request: GenerateRequest):
    """Generate text using BitNet model"""
    
    if not model_loaded:
        raise HTTPException(status_code=503, detail="Model not loaded")
    
    try:
        response, tokens_generated, generation_time = await generate_response_with_iterative_mcp(
            messages=request.messages,
            context=request.context,
            use_mcp_context=True
        )
        
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

@app.get("/")
async def root():
    """Root endpoint with API information"""
    return {
        "message": "BitNet FastAPI Server",
        "version": "1.0.0",
        "model": MODEL_NAME,
        "status": "running" if model_loaded else "loading",
        "endpoints": {
            "health": "/health",
            "generate": "/generate",
            "docs": "/docs"
        }
    }

@app.post("/mcp/tools")
async def mcp_tools(request: Dict[str, Any]):
    """Access MCP tools directly"""
    global mcp_client
    
    if not mcp_client:
        raise HTTPException(status_code=503, detail="MCP client not initialized")
    
    try:
        tool = request.get("tool")
        args = request.get("args", {})
        
        if not tool:
            raise HTTPException(status_code=400, detail="Tool name is required")
        
        logger.info(f"🔧 BitNet MCP Tool Request: {tool}")
        result = await mcp_client.call_tool(tool, args)
        
        return {
            "tool": tool,
            "args": args,
            "result": result,
            "timestamp": time.time()
        }
        
    except Exception as e:
        logger.error(f"MCP tools endpoint error: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.get("/mcp/collections")
async def mcp_collections():
    """List all collections via MCP"""
    global mcp_client
    
    if not mcp_client:
        raise HTTPException(status_code=503, detail="MCP client not initialized")
    
    try:
        collections = await mcp_client.list_collections()
        return {
            "collections": collections,
            "count": len(collections),
            "timestamp": time.time()
        }
    except Exception as e:
        logger.error(f"MCP collections endpoint error: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.get("/info")
async def model_info():
    """Get detailed model information"""
    if not model_loaded:
        raise HTTPException(status_code=503, detail="Model not loaded")
    
    return {
        "model_name": MODEL_NAME,
        "model_path": str(MODEL_PATH),
        "device": str(next(model.parameters()).device),
        "dtype": str(next(model.parameters()).dtype),
        "parameters": sum(p.numel() for p in model.parameters()),
        "config": {
            "max_tokens": MAX_TOKENS,
            "temperature": TEMPERATURE,
            "top_p": TOP_P
        }
    }

if __name__ == "__main__":
    # Run the server
    uvicorn.run(
        "bitnet_server:app",
        host="0.0.0.0",
        port=15003,
        reload=False,
        log_level="info"
    )

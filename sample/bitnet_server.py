#!/usr/bin/env python3
"""
BitNet FastAPI Server
Serves the BitNet b1.58 2B4T model via REST API
"""

import asyncio
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
    """Load the BitNet model and tokenizer"""
    global model, tokenizer, model_loaded
    
    try:
        logger.info("🔄 Loading BitNet model and tokenizer...")
        
        # Load tokenizer
        tokenizer = AutoTokenizer.from_pretrained(MODEL_NAME)
        logger.info("✅ Tokenizer loaded")
        
        # Load model
        model = AutoModelForCausalLM.from_pretrained(
            MODEL_NAME,
            torch_dtype=torch.bfloat16,
            device_map="auto" if torch.cuda.is_available() else "cpu"
        )
        logger.info("✅ Model loaded")
        
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
        formatted_messages.append({
            "role": "system",
            "content": f"You are a helpful AI assistant. Here is some relevant context: {context}"
        })
    else:
        formatted_messages.append({
            "role": "system", 
            "content": "You are a helpful AI assistant powered by BitNet, a 1-bit language model."
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

async def generate_response(
    messages: List[ChatMessage],
    context: str = "",
    max_tokens: int = MAX_TOKENS,
    temperature: float = TEMPERATURE,
    top_p: float = TOP_P
) -> tuple[str, int, float]:
    """Generate response using BitNet model"""
    
    if not model_loaded:
        raise HTTPException(status_code=503, detail="Model not loaded")
    
    start_time = time.time()
    
    try:
        # Format the prompt
        prompt = format_chat_messages(messages, context)
        
        # Tokenize input
        inputs = tokenizer(prompt, return_tensors="pt")
        
        # Move to same device as model
        device = next(model.parameters()).device
        inputs = {k: v.to(device) for k, v in inputs.items()}
        
        # Generate response
        with torch.no_grad():
            outputs = model.generate(
                **inputs,
                max_new_tokens=max_tokens,
                temperature=temperature,
                top_p=top_p,
                do_sample=True,
                pad_token_id=tokenizer.eos_token_id,
                eos_token_id=tokenizer.eos_token_id
            )
        
        # Decode response (only the new tokens)
        input_length = inputs['input_ids'].shape[1]
        response_tokens = outputs[0][input_length:]
        response = tokenizer.decode(response_tokens, skip_special_tokens=True)
        
        generation_time = time.time() - start_time
        tokens_generated = len(response_tokens)
        
        logger.info(f"Generated {tokens_generated} tokens in {generation_time:.2f}s")
        
        return response.strip(), tokens_generated, generation_time
        
    except Exception as e:
        logger.error(f"Generation error: {e}")
        raise HTTPException(status_code=500, detail=f"Generation failed: {str(e)}")

@app.on_event("startup")
async def startup_event():
    """Initialize the model on startup"""
    try:
        check_model_file()
        await load_model()
    except Exception as e:
        logger.error(f"Startup failed: {e}")
        sys.exit(1)

@app.get("/health", response_model=HealthResponse)
async def health_check():
    """Health check endpoint"""
    model_info = {
        "name": MODEL_NAME,
        "path": str(MODEL_PATH),
        "loaded": model_loaded,
        "device": str(next(model.parameters()).device) if model_loaded else "unknown"
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
        response, tokens_generated, generation_time = await generate_response(
            messages=request.messages,
            context=request.context,
            max_tokens=request.max_tokens,
            temperature=request.temperature,
            top_p=request.top_p
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
        port=8000,
        reload=False,
        log_level="info"
    )

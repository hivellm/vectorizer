# Guia Completo: Isolamento por Tenant no Vectorizer

Este guia explica como implementar isolamento de coleções por tenant usando autenticação com usuário/senha, aproveitando toda a infraestrutura pronta do Vectorizer **sem precisar alterar o código do Vectorizer**.

## 📋 Índice

1. [Como Funciona o Isolamento](#como-funciona-o-isolamento)
2. [Arquitetura](#arquitetura)
3. [Node.js - Implementação Completa](#nodejs---implementação-completa)
4. [Golang - Implementação Completa](#golang---implementação-completa)
5. [Exemplos de Uso](#exemplos-de-uso)
6. [Processamento de Documentos e Conversão em Vetores](#processamento-de-documentos-e-conversão-em-vetores)
7. [Integração com MCP](#integração-com-mcp)
8. [Boas Práticas](#boas-práticas)
9. [Troubleshooting](#troubleshooting)

---

## Como Funciona o Isolamento

### Conceito

O Vectorizer usa **RequestTenantContext** para isolar coleções por tenant. Quando você envia o header `X-HiveHub-User-ID` com um UUID de tenant, o sistema:

1. **Cria automaticamente um TenantContext** no middleware
2. **Associa coleções ao tenant_id** ao criá-las
3. **Filtra automaticamente** todas as operações para mostrar apenas coleções do tenant
4. **Bloqueia acesso** a coleções de outros tenants (retorna 404)

### Headers Necessários

Para ativar o isolamento, envie estes headers em todas as requisições:

```http
X-HiveHub-User-ID: <uuid-do-tenant>
X-HiveHub-Service: <nome-do-seu-app>
```

**Por que funciona:**
- `X-HiveHub-Service` identifica como request interno (bypass de autenticação HiveHub)
- `X-HiveHub-User-ID` é extraído e convertido em `TenantContext`
- O sistema trata como se fosse uma API key do HiveHub, mas você controla o tenant_id

---

## Arquitetura

```
┌─────────────────┐
│  Seu Sistema    │
│  (Node/Go)      │
└────────┬────────┘
         │
         │ 1. Autentica usuário/senha
         │ 2. Busca tenant_id do usuário
         │
         ▼
┌─────────────────┐
│  Vectorizer     │
│  REST API       │
└────────┬────────┘
         │
         │ 3. Middleware extrai X-HiveHub-User-ID
         │ 4. Cria TenantContext
         │
         ▼
┌─────────────────┐
│  VectorStore    │
│  (Isolamento)   │
└─────────────────┘
```

---

## Node.js - Implementação Completa

### 1. Instalação de Dependências

```bash
npm install axios uuid
# ou
yarn add axios uuid
```

### 2. Cliente Vectorizer com Isolamento

```typescript
// vectorizer-client.ts
import axios, { AxiosInstance } from 'axios';
import { v4 as uuidv4 } from 'uuid';

export interface VectorizerConfig {
  baseUrl: string;
  tenantId: string;
  serviceName?: string;
}

export interface CollectionConfig {
  name: string;
  dimension: number;
  metric?: 'cosine' | 'euclidean' | 'dot';
  graph?: {
    enabled: boolean;
  };
}

export interface Vector {
  id: string;
  data: number[];
  payload?: Record<string, any>;
}

export interface SearchRequest {
  query: string;
  limit?: number;
  collection?: string;
}

export class VectorizerClient {
  private client: AxiosInstance;
  private tenantId: string;
  private serviceName: string;

  constructor(config: VectorizerConfig) {
    this.tenantId = config.tenantId;
    this.serviceName = config.serviceName || 'custom-app';
    
    this.client = axios.create({
      baseURL: config.baseUrl,
      headers: {
        'Content-Type': 'application/json',
        // Headers para isolamento por tenant
        'X-HiveHub-User-ID': this.tenantId,
        'X-HiveHub-Service': this.serviceName,
      },
    });
  }

  /**
   * Criar uma nova coleção (isolada para este tenant)
   */
  async createCollection(config: CollectionConfig): Promise<any> {
    try {
      const response = await this.client.post('/api/v1/collections', {
        name: config.name,
        dimension: config.dimension,
        metric: config.metric || 'cosine',
        graph: config.graph,
      });
      return response.data;
    } catch (error: any) {
      throw new Error(`Failed to create collection: ${error.response?.data?.error || error.message}`);
    }
  }

  /**
   * Listar todas as coleções do tenant (só vê as deste tenant)
   */
  async listCollections(): Promise<string[]> {
    try {
      const response = await this.client.get('/api/v1/collections');
      return response.data.collections || [];
    } catch (error: any) {
      throw new Error(`Failed to list collections: ${error.response?.data?.error || error.message}`);
    }
  }

  /**
   * Obter informações de uma coleção (só se pertencer ao tenant)
   */
  async getCollectionInfo(collectionName: string): Promise<any> {
    try {
      const response = await this.client.get(`/api/v1/collections/${collectionName}`);
      return response.data;
    } catch (error: any) {
      if (error.response?.status === 404) {
        throw new Error(`Collection '${collectionName}' not found or not accessible`);
      }
      throw new Error(`Failed to get collection info: ${error.response?.data?.error || error.message}`);
    }
  }

  /**
   * Inserir vetores em uma coleção (só se pertencer ao tenant)
   */
  async insertVectors(collectionName: string, vectors: Vector[]): Promise<any> {
    try {
      const response = await this.client.post(`/api/v1/collections/${collectionName}/vectors`, {
        vectors: vectors.map(v => ({
          id: v.id,
          vector: v.data,
          payload: v.payload,
        })),
      });
      return response.data;
    } catch (error: any) {
      throw new Error(`Failed to insert vectors: ${error.response?.data?.error || error.message}`);
    }
  }

  /**
   * Buscar vetores similares (só em coleções do tenant)
   */
  async search(collectionName: string, query: SearchRequest): Promise<any> {
    try {
      const response = await this.client.post(`/api/v1/collections/${collectionName}/search`, {
        query: query.query,
        limit: query.limit || 10,
      });
      return response.data;
    } catch (error: any) {
      throw new Error(`Failed to search: ${error.response?.data?.error || error.message}`);
    }
  }

  /**
   * Busca inteligente (só em coleções do tenant)
   */
  async intelligentSearch(query: string, collections?: string[], maxResults?: number): Promise<any> {
    try {
      const response = await this.client.post('/api/v1/search/intelligent', {
        query,
        collections,
        max_results: maxResults || 10,
        mmr_enabled: false,
        domain_expansion: false,
        technical_focus: true,
      });
      return response.data;
    } catch (error: any) {
      throw new Error(`Failed to intelligent search: ${error.response?.data?.error || error.message}`);
    }
  }

  /**
   * Deletar uma coleção (só se pertencer ao tenant)
   */
  async deleteCollection(collectionName: string): Promise<void> {
    try {
      await this.client.delete(`/api/v1/collections/${collectionName}`);
    } catch (error: any) {
      throw new Error(`Failed to delete collection: ${error.response?.data?.error || error.message}`);
    }
  }
}
```

### 3. Sistema de Autenticação e Tenant

```typescript
// auth-service.ts
import { v4 as uuidv4 } from 'uuid';

export interface User {
  id: string;
  username: string;
  passwordHash: string;
  tenantId: string;
  tenantName: string;
}

export class AuthService {
  private users: Map<string, User> = new Map();

  /**
   * Registrar um novo usuário com tenant
   */
  async registerUser(
    username: string,
    password: string,
    tenantId: string,
    tenantName: string
  ): Promise<User> {
    const passwordHash = await this.hashPassword(password);
    const user: User = {
      id: uuidv4(),
      username,
      passwordHash,
      tenantId,
      tenantName,
    };
    this.users.set(username, user);
    return user;
  }

  /**
   * Autenticar usuário e retornar informações do tenant
   */
  async authenticate(username: string, password: string): Promise<{ tenantId: string; tenantName: string } | null> {
    const user = this.users.get(username);
    if (!user) {
      return null;
    }

    const isValid = await this.verifyPassword(password, user.passwordHash);
    if (!isValid) {
      return null;
    }

    return {
      tenantId: user.tenantId,
      tenantName: user.tenantName,
    };
  }

  /**
   * Obter tenant_id de um usuário autenticado
   */
  async getTenantId(username: string): Promise<string | null> {
    const user = this.users.get(username);
    return user?.tenantId || null;
  }

  private async hashPassword(password: string): Promise<string> {
    // Em produção, use bcrypt ou similar
    const crypto = await import('crypto');
    return crypto.createHash('sha256').update(password).digest('hex');
  }

  private async verifyPassword(password: string, hash: string): Promise<boolean> {
    const crypto = await import('crypto');
    const passwordHash = crypto.createHash('sha256').update(password).digest('hex');
    return passwordHash === hash;
  }
}
```

### 4. Exemplo de Uso Completo

```typescript
// example.ts
import { VectorizerClient } from './vectorizer-client';
import { AuthService } from './auth-service';

async function main() {
  // 1. Inicializar serviços
  const authService = new AuthService();
  const vectorizerUrl = 'http://localhost:15002';

  // 2. Registrar tenants e usuários
  const tenantAId = '550e8400-e29b-41d4-a716-446655440000';
  const tenantBId = '660e8400-e29b-41d4-a716-446655440001';

  await authService.registerUser(
    'usuario_a',
    'senha123',
    tenantAId,
    'Tenant A'
  );

  await authService.registerUser(
    'usuario_b',
    'senha456',
    tenantBId,
    'Tenant B'
  );

  // 3. Autenticar Tenant A
  const tenantA = await authService.authenticate('usuario_a', 'senha123');
  if (!tenantA) {
    console.error('Autenticação falhou');
    return;
  }

  // 4. Criar cliente Vectorizer para Tenant A
  const clientA = new VectorizerClient({
    baseUrl: vectorizerUrl,
    tenantId: tenantA.tenantId,
    serviceName: 'meu-app',
  });

  // 5. Criar coleção para Tenant A
  console.log('Criando coleção para Tenant A...');
  await clientA.createCollection({
    name: 'documentos',
    dimension: 384,
    metric: 'cosine',
  });

  // 6. Inserir vetores
  console.log('Inserindo vetores...');
  await clientA.insertVectors('documentos', [
    {
      id: 'doc1',
      data: Array(384).fill(0).map(() => Math.random()),
      payload: { title: 'Documento 1', content: 'Conteúdo do documento 1' },
    },
    {
      id: 'doc2',
      data: Array(384).fill(0).map(() => Math.random()),
      payload: { title: 'Documento 2', content: 'Conteúdo do documento 2' },
    },
  ]);

  // 7. Listar coleções (só vê as do Tenant A)
  console.log('Listando coleções do Tenant A...');
  const collectionsA = await clientA.listCollections();
  console.log('Coleções do Tenant A:', collectionsA);

  // 8. Buscar
  console.log('Buscando documentos...');
  const results = await clientA.search('documentos', {
    query: 'documento',
    limit: 5,
  });
  console.log('Resultados:', results);

  // 9. Autenticar Tenant B
  const tenantB = await authService.authenticate('usuario_b', 'senha456');
  if (!tenantB) {
    console.error('Autenticação falhou');
    return;
  }

  // 10. Criar cliente para Tenant B
  const clientB = new VectorizerClient({
    baseUrl: vectorizerUrl,
    tenantId: tenantB.tenantId,
    serviceName: 'meu-app',
  });

  // 11. Tentar acessar coleção do Tenant A (deve falhar)
  console.log('\nTentando acessar coleção do Tenant A com Tenant B...');
  try {
    await clientB.getCollectionInfo('documentos');
    console.error('ERRO: Tenant B conseguiu acessar coleção do Tenant A!');
  } catch (error: any) {
    console.log('✅ Isolamento funcionando:', error.message);
  }

  // 12. Criar coleção própria para Tenant B
  console.log('\nCriando coleção para Tenant B...');
  await clientB.createCollection({
    name: 'documentos', // Mesmo nome, mas coleção diferente!
    dimension: 384,
  });

  // 13. Listar coleções do Tenant B
  console.log('Listando coleções do Tenant B...');
  const collectionsB = await clientB.listCollections();
  console.log('Coleções do Tenant B:', collectionsB);

  // 14. Verificar isolamento
  console.log('\n✅ Isolamento verificado:');
  console.log(`  Tenant A tem ${collectionsA.length} coleção(ões)`);
  console.log(`  Tenant B tem ${collectionsB.length} coleção(ões)`);
  console.log('  Cada tenant só vê suas próprias coleções!');
}

main().catch(console.error);
```

### 5. Integração com Express.js

```typescript
// server.ts
import express from 'express';
import { VectorizerClient } from './vectorizer-client';
import { AuthService } from './auth-service';

const app = express();
app.use(express.json());

const authService = new AuthService();
const vectorizerUrl = 'http://localhost:15002';

// Middleware de autenticação
async function authenticate(req: express.Request, res: express.Response, next: express.NextFunction) {
  const { username, password } = req.body;
  
  if (!username || !password) {
    return res.status(401).json({ error: 'Username and password required' });
  }

  const tenant = await authService.authenticate(username, password);
  if (!tenant) {
    return res.status(401).json({ error: 'Invalid credentials' });
  }

  // Adicionar tenant ao request
  (req as any).tenant = tenant;
  next();
}

// Endpoint de login
app.post('/api/login', async (req, res) => {
  const { username, password } = req.body;
  const tenant = await authService.authenticate(username, password);
  
  if (!tenant) {
    return res.status(401).json({ error: 'Invalid credentials' });
  }

  res.json({ tenantId: tenant.tenantId, tenantName: tenant.tenantName });
});

// Endpoint para criar coleção (requer autenticação)
app.post('/api/collections', authenticate, async (req, res) => {
  try {
    const tenant = (req as any).tenant;
    const client = new VectorizerClient({
      baseUrl: vectorizerUrl,
      tenantId: tenant.tenantId,
    });

    const collection = await client.createCollection(req.body);
    res.json(collection);
  } catch (error: any) {
    res.status(500).json({ error: error.message });
  }
});

// Endpoint para listar coleções (requer autenticação)
app.get('/api/collections', authenticate, async (req, res) => {
  try {
    const tenant = (req as any).tenant;
    const client = new VectorizerClient({
      baseUrl: vectorizerUrl,
      tenantId: tenant.tenantId,
    });

    const collections = await client.listCollections();
    res.json({ collections });
  } catch (error: any) {
    res.status(500).json({ error: error.message });
  }
});

app.listen(3000, () => {
  console.log('Server running on http://localhost:3000');
});
```

---

## Golang - Implementação Completa

### 1. Estrutura do Projeto

```
go mod init vectorizer-tenant-client
```

### 2. Cliente Vectorizer com Isolamento

```go
// vectorizer/client.go
package vectorizer

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"time"
)

// Config configuração do cliente
type Config struct {
	BaseURL    string
	TenantID   string
	ServiceName string
}

// Client cliente Vectorizer com isolamento por tenant
type Client struct {
	baseURL     string
	tenantID    string
	serviceName string
	httpClient  *http.Client
}

// NewClient cria um novo cliente Vectorizer
func NewClient(config Config) *Client {
	return &Client{
		baseURL:     config.BaseURL,
		tenantID:    config.TenantID,
		serviceName: config.ServiceName,
		httpClient: &http.Client{
			Timeout: 30 * time.Second,
		},
	}
}

// CollectionConfig configuração para criar coleção
type CollectionConfig struct {
	Name      string `json:"name"`
	Dimension int    `json:"dimension"`
	Metric    string `json:"metric,omitempty"`
	Graph     *GraphConfig `json:"graph,omitempty"`
}

type GraphConfig struct {
	Enabled bool `json:"enabled"`
}

// Vector representa um vetor
type Vector struct {
	ID      string                 `json:"id"`
	Data    []float32              `json:"vector"`
	Payload map[string]interface{} `json:"payload,omitempty"`
}

// SearchRequest requisição de busca
type SearchRequest struct {
	Query    string `json:"query"`
	Limit    int    `json:"limit,omitempty"`
	Collection string `json:"collection,omitempty"`
}

// makeRequest faz uma requisição HTTP com headers de tenant
func (c *Client) makeRequest(method, endpoint string, body interface{}) (*http.Response, error) {
	var reqBody io.Reader
	if body != nil {
		jsonData, err := json.Marshal(body)
		if err != nil {
			return nil, fmt.Errorf("failed to marshal request body: %w", err)
		}
		reqBody = bytes.NewBuffer(jsonData)
	}

	req, err := http.NewRequest(method, c.baseURL+endpoint, reqBody)
	if err != nil {
		return nil, fmt.Errorf("failed to create request: %w", err)
	}

	// Headers para isolamento por tenant
	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("X-HiveHub-User-ID", c.tenantID)
	req.Header.Set("X-HiveHub-Service", c.serviceName)

	resp, err := c.httpClient.Do(req)
	if err != nil {
		return nil, fmt.Errorf("request failed: %w", err)
	}

	return resp, nil
}

// CreateCollection cria uma nova coleção (isolada para este tenant)
func (c *Client) CreateCollection(config CollectionConfig) (map[string]interface{}, error) {
	if config.Metric == "" {
		config.Metric = "cosine"
	}

	resp, err := c.makeRequest("POST", "/api/v1/collections", config)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK && resp.StatusCode != http.StatusCreated {
		body, _ := io.ReadAll(resp.Body)
		return nil, fmt.Errorf("failed to create collection: %s", string(body))
	}

	var result map[string]interface{}
	if err := json.NewDecoder(resp.Body).Decode(&result); err != nil {
		return nil, fmt.Errorf("failed to decode response: %w", err)
	}

	return result, nil
}

// ListCollections lista todas as coleções do tenant (só vê as deste tenant)
func (c *Client) ListCollections() ([]string, error) {
	resp, err := c.makeRequest("GET", "/api/v1/collections", nil)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		body, _ := io.ReadAll(resp.Body)
		return nil, fmt.Errorf("failed to list collections: %s", string(body))
	}

	var result struct {
		Collections []string `json:"collections"`
	}
	if err := json.NewDecoder(resp.Body).Decode(&result); err != nil {
		return nil, fmt.Errorf("failed to decode response: %w", err)
	}

	return result.Collections, nil
}

// GetCollectionInfo obtém informações de uma coleção (só se pertencer ao tenant)
func (c *Client) GetCollectionInfo(collectionName string) (map[string]interface{}, error) {
	resp, err := c.makeRequest("GET", fmt.Sprintf("/api/v1/collections/%s", collectionName), nil)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()

	if resp.StatusCode == http.StatusNotFound {
		return nil, fmt.Errorf("collection '%s' not found or not accessible", collectionName)
	}

	if resp.StatusCode != http.StatusOK {
		body, _ := io.ReadAll(resp.Body)
		return nil, fmt.Errorf("failed to get collection info: %s", string(body))
	}

	var result map[string]interface{}
	if err := json.NewDecoder(resp.Body).Decode(&result); err != nil {
		return nil, fmt.Errorf("failed to decode response: %w", err)
	}

	return result, nil
}

// InsertVectors insere vetores em uma coleção (só se pertencer ao tenant)
func (c *Client) InsertVectors(collectionName string, vectors []Vector) (map[string]interface{}, error) {
	payload := map[string]interface{}{
		"vectors": vectors,
	}

	resp, err := c.makeRequest("POST", fmt.Sprintf("/api/v1/collections/%s/vectors", collectionName), payload)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK && resp.StatusCode != http.StatusCreated {
		body, _ := io.ReadAll(resp.Body)
		return nil, fmt.Errorf("failed to insert vectors: %s", string(body))
	}

	var result map[string]interface{}
	if err := json.NewDecoder(resp.Body).Decode(&result); err != nil {
		return nil, fmt.Errorf("failed to decode response: %w", err)
	}

	return result, nil
}

// Search busca vetores similares (só em coleções do tenant)
func (c *Client) Search(collectionName string, query SearchRequest) (map[string]interface{}, error) {
	payload := map[string]interface{}{
		"query": query.Query,
		"limit": query.Limit,
	}
	if query.Limit == 0 {
		payload["limit"] = 10
	}

	resp, err := c.makeRequest("POST", fmt.Sprintf("/api/v1/collections/%s/search", collectionName), payload)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		body, _ := io.ReadAll(resp.Body)
		return nil, fmt.Errorf("failed to search: %s", string(body))
	}

	var result map[string]interface{}
	if err := json.NewDecoder(resp.Body).Decode(&result); err != nil {
		return nil, fmt.Errorf("failed to decode response: %w", err)
	}

	return result, nil
}

// IntelligentSearch busca inteligente (só em coleções do tenant)
func (c *Client) IntelligentSearch(query string, collections []string, maxResults int) (map[string]interface{}, error) {
	payload := map[string]interface{}{
		"query":            query,
		"collections":      collections,
		"max_results":      maxResults,
		"mmr_enabled":      false,
		"domain_expansion": false,
		"technical_focus":  true,
	}

	resp, err := c.makeRequest("POST", "/api/v1/search/intelligent", payload)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		body, _ := io.ReadAll(resp.Body)
		return nil, fmt.Errorf("failed to intelligent search: %s", string(body))
	}

	var result map[string]interface{}
	if err := json.NewDecoder(resp.Body).Decode(&result); err != nil {
		return nil, fmt.Errorf("failed to decode response: %w", err)
	}

	return result, nil
}

// DeleteCollection deleta uma coleção (só se pertencer ao tenant)
func (c *Client) DeleteCollection(collectionName string) error {
	resp, err := c.makeRequest("DELETE", fmt.Sprintf("/api/v1/collections/%s", collectionName), nil)
	if err != nil {
		return err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK && resp.StatusCode != http.StatusNoContent {
		body, _ := io.ReadAll(resp.Body)
		return fmt.Errorf("failed to delete collection: %s", string(body))
	}

	return nil
}
```

### 3. Sistema de Autenticação e Tenant

```go
// auth/service.go
package auth

import (
	"crypto/sha256"
	"encoding/hex"
	"errors"
	"sync"

	"github.com/google/uuid"
)

// User representa um usuário
type User struct {
	ID          string
	Username    string
	PasswordHash string
	TenantID    string
	TenantName  string
}

// TenantInfo informações do tenant
type TenantInfo struct {
	TenantID   string
	TenantName string
}

// Service serviço de autenticação
type Service struct {
	users map[string]*User
	mu    sync.RWMutex
}

// NewService cria um novo serviço de autenticação
func NewService() *Service {
	return &Service{
		users: make(map[string]*User),
	}
}

// RegisterUser registra um novo usuário com tenant
func (s *Service) RegisterUser(username, password, tenantID, tenantName string) (*User, error) {
	s.mu.Lock()
	defer s.mu.Unlock()

	if _, exists := s.users[username]; exists {
		return nil, errors.New("user already exists")
	}

	passwordHash := hashPassword(password)
	user := &User{
		ID:           uuid.New().String(),
		Username:     username,
		PasswordHash: passwordHash,
		TenantID:     tenantID,
		TenantName:   tenantName,
	}

	s.users[username] = user
	return user, nil
}

// Authenticate autentica usuário e retorna informações do tenant
func (s *Service) Authenticate(username, password string) (*TenantInfo, error) {
	s.mu.RLock()
	defer s.mu.RUnlock()

	user, exists := s.users[username]
	if !exists {
		return nil, errors.New("invalid credentials")
	}

	if !verifyPassword(password, user.PasswordHash) {
		return nil, errors.New("invalid credentials")
	}

	return &TenantInfo{
		TenantID:   user.TenantID,
		TenantName: user.TenantName,
	}, nil
}

// GetTenantID obtém tenant_id de um usuário autenticado
func (s *Service) GetTenantID(username string) (string, error) {
	s.mu.RLock()
	defer s.mu.RUnlock()

	user, exists := s.users[username]
	if !exists {
		return "", errors.New("user not found")
	}

	return user.TenantID, nil
}

func hashPassword(password string) string {
	hash := sha256.Sum256([]byte(password))
	return hex.EncodeToString(hash[:])
}

func verifyPassword(password, hash string) bool {
	passwordHash := hashPassword(password)
	return passwordHash == hash
}
```

### 4. Exemplo de Uso Completo

```go
// main.go
package main

import (
	"fmt"
	"log"

	"github.com/google/uuid"
	"vectorizer-tenant-client/auth"
	"vectorizer-tenant-client/vectorizer"
)

func main() {
	// 1. Inicializar serviços
	authService := auth.NewService()
	vectorizerURL := "http://localhost:15002"

	// 2. Registrar tenants e usuários
	tenantAID := "550e8400-e29b-41d4-a716-446655440000"
	tenantBID := "660e8400-e29b-41d4-a716-446655440001"

	_, err := authService.RegisterUser("usuario_a", "senha123", tenantAID, "Tenant A")
	if err != nil {
		log.Fatal(err)
	}

	_, err = authService.RegisterUser("usuario_b", "senha456", tenantBID, "Tenant B")
	if err != nil {
		log.Fatal(err)
	}

	// 3. Autenticar Tenant A
	tenantA, err := authService.Authenticate("usuario_a", "senha123")
	if err != nil {
		log.Fatal(err)
	}

	// 4. Criar cliente Vectorizer para Tenant A
	clientA := vectorizer.NewClient(vectorizer.Config{
		BaseURL:     vectorizerURL,
		TenantID:    tenantA.TenantID,
		ServiceName: "meu-app",
	})

	// 5. Criar coleção para Tenant A
	fmt.Println("Criando coleção para Tenant A...")
	_, err = clientA.CreateCollection(vectorizer.CollectionConfig{
		Name:      "documentos",
		Dimension: 384,
		Metric:    "cosine",
	})
	if err != nil {
		log.Fatal(err)
	}

	// 6. Inserir vetores
	fmt.Println("Inserindo vetores...")
	vectors := []vectorizer.Vector{
		{
			ID:   "doc1",
			Data: generateRandomVector(384),
			Payload: map[string]interface{}{
				"title":   "Documento 1",
				"content": "Conteúdo do documento 1",
			},
		},
		{
			ID:   "doc2",
			Data: generateRandomVector(384),
			Payload: map[string]interface{}{
				"title":   "Documento 2",
				"content": "Conteúdo do documento 2",
			},
		},
	}
	_, err = clientA.InsertVectors("documentos", vectors)
	if err != nil {
		log.Fatal(err)
	}

	// 7. Listar coleções (só vê as do Tenant A)
	fmt.Println("Listando coleções do Tenant A...")
	collectionsA, err := clientA.ListCollections()
	if err != nil {
		log.Fatal(err)
	}
	fmt.Printf("Coleções do Tenant A: %v\n", collectionsA)

	// 8. Buscar
	fmt.Println("Buscando documentos...")
	results, err := clientA.Search("documentos", vectorizer.SearchRequest{
		Query: "documento",
		Limit: 5,
	})
	if err != nil {
		log.Fatal(err)
	}
	fmt.Printf("Resultados: %+v\n", results)

	// 9. Autenticar Tenant B
	tenantB, err := authService.Authenticate("usuario_b", "senha456")
	if err != nil {
		log.Fatal(err)
	}

	// 10. Criar cliente para Tenant B
	clientB := vectorizer.NewClient(vectorizer.Config{
		BaseURL:     vectorizerURL,
		TenantID:    tenantB.TenantID,
		ServiceName: "meu-app",
	})

	// 11. Tentar acessar coleção do Tenant A (deve falhar)
	fmt.Println("\nTentando acessar coleção do Tenant A com Tenant B...")
	_, err = clientB.GetCollectionInfo("documentos")
	if err != nil {
		fmt.Printf("✅ Isolamento funcionando: %v\n", err)
	} else {
		fmt.Println("ERRO: Tenant B conseguiu acessar coleção do Tenant A!")
	}

	// 12. Criar coleção própria para Tenant B
	fmt.Println("\nCriando coleção para Tenant B...")
	_, err = clientB.CreateCollection(vectorizer.CollectionConfig{
		Name:      "documentos", // Mesmo nome, mas coleção diferente!
		Dimension: 384,
	})
	if err != nil {
		log.Fatal(err)
	}

	// 13. Listar coleções do Tenant B
	fmt.Println("Listando coleções do Tenant B...")
	collectionsB, err := clientB.ListCollections()
	if err != nil {
		log.Fatal(err)
	}
	fmt.Printf("Coleções do Tenant B: %v\n", collectionsB)

	// 14. Verificar isolamento
	fmt.Println("\n✅ Isolamento verificado:")
	fmt.Printf("  Tenant A tem %d coleção(ões)\n", len(collectionsA))
	fmt.Printf("  Tenant B tem %d coleção(ões)\n", len(collectionsB))
	fmt.Println("  Cada tenant só vê suas próprias coleções!")
}

func generateRandomVector(dimension int) []float32 {
	vector := make([]float32, dimension)
	for i := range vector {
		vector[i] = float32(i) / float32(dimension) // Exemplo simples
	}
	return vector
}
```

### 5. Integração com Gin (HTTP Framework)

```go
// server.go
package main

import (
	"net/http"

	"github.com/gin-gonic/gin"
	"vectorizer-tenant-client/auth"
	"vectorizer-tenant-client/vectorizer"
)

type Server struct {
	authService   *auth.Service
	vectorizerURL string
}

func NewServer(authService *auth.Service, vectorizerURL string) *Server {
	return &Server{
		authService:   authService,
		vectorizerURL: vectorizerURL,
	}
}

// Middleware de autenticação
func (s *Server) authenticateMiddleware(c *gin.Context) {
	var loginReq struct {
		Username string `json:"username"`
		Password string `json:"password"`
	}

	// Em produção, use header Authorization ou cookie
	if err := c.ShouldBindJSON(&loginReq); err != nil {
		c.JSON(http.StatusUnauthorized, gin.H{"error": "Username and password required"})
		c.Abort()
		return
	}

	tenant, err := s.authService.Authenticate(loginReq.Username, loginReq.Password)
	if err != nil {
		c.JSON(http.StatusUnauthorized, gin.H{"error": "Invalid credentials"})
		c.Abort()
		return
	}

	// Adicionar tenant ao context
	c.Set("tenant", tenant)
	c.Next()
}

// Endpoint de login
func (s *Server) login(c *gin.Context) {
	var loginReq struct {
		Username string `json:"username"`
		Password string `json:"password"`
	}

	if err := c.ShouldBindJSON(&loginReq); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid request"})
		return
	}

	tenant, err := s.authService.Authenticate(loginReq.Username, loginReq.Password)
	if err != nil {
		c.JSON(http.StatusUnauthorized, gin.H{"error": "Invalid credentials"})
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"tenantId":   tenant.TenantID,
		"tenantName": tenant.TenantName,
	})
}

// Endpoint para criar coleção
func (s *Server) createCollection(c *gin.Context) {
	tenant, exists := c.Get("tenant")
	if !exists {
		c.JSON(http.StatusUnauthorized, gin.H{"error": "Not authenticated"})
		return
	}

	tenantInfo := tenant.(*auth.TenantInfo)
	client := vectorizer.NewClient(vectorizer.Config{
		BaseURL:     s.vectorizerURL,
		TenantID:    tenantInfo.TenantID,
		ServiceName: "meu-app",
	})

	var config vectorizer.CollectionConfig
	if err := c.ShouldBindJSON(&config); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid request"})
		return
	}

	result, err := client.CreateCollection(config)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusOK, result)
}

// Endpoint para listar coleções
func (s *Server) listCollections(c *gin.Context) {
	tenant, exists := c.Get("tenant")
	if !exists {
		c.JSON(http.StatusUnauthorized, gin.H{"error": "Not authenticated"})
		return
	}

	tenantInfo := tenant.(*auth.TenantInfo)
	client := vectorizer.NewClient(vectorizer.Config{
		BaseURL:     s.vectorizerURL,
		TenantID:    tenantInfo.TenantID,
		ServiceName: "meu-app",
	})

	collections, err := client.ListCollections()
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusOK, gin.H{"collections": collections})
}

func main() {
	authService := auth.NewService()
	server := NewServer(authService, "http://localhost:15002")

	r := gin.Default()

	// Rotas públicas
	r.POST("/api/login", server.login)

	// Rotas protegidas
	protected := r.Group("/api")
	protected.Use(server.authenticateMiddleware)
	{
		protected.POST("/collections", server.createCollection)
		protected.GET("/collections", server.listCollections)
	}

	r.Run(":3000")
}
```

---

## Exemplos de Uso

### Cenário 1: SaaS Multi-Tenant

```typescript
// Cada cliente da sua SaaS é um tenant
const clientes = [
  { id: 'tenant-1', nome: 'Empresa A' },
  { id: 'tenant-2', nome: 'Empresa B' },
];

// Cada cliente só vê suas próprias coleções
for (const cliente of clientes) {
  const client = new VectorizerClient({
    baseUrl: 'http://localhost:15002',
    tenantId: cliente.id,
  });
  
  // Operações isoladas automaticamente
  await client.createCollection({ name: 'docs', dimension: 384 });
}
```

### Cenário 2: Aplicação com Usuários

```go
// Cada usuário pertence a um tenant
type User struct {
    ID       string
    Username string
    TenantID string
}

// Ao fazer login, criar cliente com tenant_id do usuário
func handleLogin(username, password string) (*vectorizer.Client, error) {
    user := getUserByUsername(username)
    tenantID := user.TenantID
    
    client := vectorizer.NewClient(vectorizer.Config{
        BaseURL:     "http://localhost:15002",
        TenantID:    tenantID,
        ServiceName: "my-app",
    })
    
    return client, nil
}
```

---

## Integração com MCP

### Opção 1: Usar REST API via MCP (Recomendado)

Se você usa MCP via HTTP (StreamableHTTP), pode passar os headers:

```http
POST /mcp/call-tool
Content-Type: application/json
X-HiveHub-User-ID: 550e8400-e29b-41d4-a716-446655440000
X-HiveHub-Service: mcp-client

{
  "tool": "create_collection",
  "arguments": {
    "name": "minha-colecao",
    "dimension": 384
  }
}
```

### Opção 2: Passar tenant_id nos Argumentos

Você pode adicionar `tenant_id` como parâmetro em cada chamada MCP:

```json
{
  "tool": "create_collection",
  "arguments": {
    "name": "minha-colecao",
    "dimension": 384,
    "tenant_id": "550e8400-e29b-41d4-a716-446655440000"
  }
}
```

**Nota:** Isso requer modificação nos handlers MCP para usar o `tenant_id` dos argumentos.

---

## Processamento de Documentos e Conversão em Vetores

O Vectorizer possui dois endpoints para processar documentos:

### Endpoint `/files/upload` (Limite: 2MB)

**Suporta transmutation automático** - converte PDF, DOCX, XLSX, PPTX, HTML, XML e imagens (com OCR) para Markdown automaticamente:

1. **Converte para Markdown** usando transmutation (PDF, DOCX, XLSX, PPTX, HTML, XML, imagens)
2. **Extrai o texto** do arquivo
3. **Divide em chunks** (pedaços menores de texto)
4. **Gera embeddings** (converte cada chunk em vetor)
5. **Armazena na coleção** (com metadados completos, incluindo informações de páginas)

**⚠️ Limitação:** Limite de 2MB devido ao Axum (framework HTTP).

### Endpoint `/api/v1/insert` (Sem limite de tamanho)

**NÃO suporta transmutation automático** - aceita apenas texto:

1. **Você precisa converter** o arquivo para Markdown no cliente (se for PDF, DOCX, etc.)
2. **Você divide em chunks** manualmente
3. **Envia cada chunk** via `/api/v1/insert`
4. **O Vectorizer gera embeddings** e armazena

**✅ Vantagem:** Sem limite de tamanho, funciona para arquivos grandes.

### Quando Usar Cada Endpoint

| Situação | Endpoint | Transmutation |
|----------|----------|---------------|
| PDF/DOCX < 2MB | `/files/upload` | ✅ Automático |
| PDF/DOCX > 2MB | `/api/v1/insert` | ⚠️ Manual (no cliente) |
| TXT/MD/Código (qualquer tamanho) | `/api/v1/insert` | ❌ Não precisa |
| HTML/XML < 2MB | `/files/upload` | ✅ Automático |
| HTML/XML > 2MB | `/api/v1/insert` | ⚠️ Manual (no cliente) |

Tudo isso **com isolamento por tenant** - cada tenant só processa e armazena em suas próprias coleções.

### Endpoint de Upload (`/files/upload`)

```
POST /files/upload
Content-Type: multipart/form-data
X-HiveHub-User-ID: <tenant-id>
X-HiveHub-Service: <service-name>
```

**✅ Suporta Transmutation Automático:**
- PDF → Markdown (com divisão por páginas)
- DOCX → Markdown (preserva estrutura, tabelas, listas)
- XLSX → Markdown (converte planilhas para tabelas)
- PPTX → Markdown (slides como páginas)
- HTML/XML → Markdown (preserva estrutura)
- Imagens (JPG, PNG, TIFF, etc.) → Markdown (com OCR)

**⚠️ IMPORTANTE:** O Axum tem um limite padrão de **2MB** para multipart. Para arquivos maiores, use o endpoint `/api/v1/insert` com conversão manual no cliente (veja [Troubleshooting de Upload](./FILE_UPLOAD_TROUBLESHOOTING.md)).

**Campos do formulário:**
- `file`: Arquivo a ser processado (obrigatório) - **Máximo 2MB**
- `collection_name`: Nome da coleção de destino (obrigatório)
- `chunk_size`: Tamanho dos chunks em caracteres (opcional)
- `chunk_overlap`: Sobreposição entre chunks (opcional)
- `metadata`: JSON com metadados adicionais (opcional)
- `public_key`: Chave pública para criptografia do payload (opcional)

**Nota:** O transmutation é aplicado automaticamente quando o arquivo é de um formato suportado. Não é necessário fazer nada especial - apenas envie o arquivo!

### Endpoint de Insert (`/api/v1/insert`)

```
POST /api/v1/insert
Content-Type: application/json
X-HiveHub-User-ID: <tenant-id>
X-HiveHub-Service: <service-name>
```

**❌ NÃO suporta transmutation automático** - aceita apenas texto já processado.

**Campos do JSON:**
- `collection`: Nome da coleção de destino (obrigatório)
- `text`: Texto a ser processado (obrigatório) - **Sem limite de tamanho**
- `metadata`: Objeto com metadados adicionais (opcional)
- `public_key`: Chave pública para criptografia do payload (opcional)

**Quando usar:**
- ✅ Arquivos de texto simples (TXT, MD, código) de qualquer tamanho
- ✅ Arquivos grandes (> 2MB) que você já converteu para Markdown no cliente
- ✅ Quando você precisa de controle total sobre o chunking

### Node.js - Upload de Documentos

```typescript
// document-processor.ts
import axios from 'axios';
import FormData from 'form-data';
import fs from 'fs';
import path from 'path';
import { VectorizerClient } from './vectorizer-client';

export interface DocumentUploadOptions {
  filePath: string;
  collectionName: string;
  chunkSize?: number;
  chunkOverlap?: number;
  metadata?: Record<string, any>;
  publicKey?: string;
}

export class DocumentProcessor {
  private client: VectorizerClient;

  constructor(client: VectorizerClient) {
    this.client = client;
  }

  /**
   * Processar e enviar documento para coleção
   * 
   * Para arquivos < 2MB que precisam de transmutation (PDF, DOCX, etc.):
   * - Usa /files/upload com transmutation automático
   * 
   * Para arquivos > 2MB ou texto simples:
   * - Usa /api/v1/insert (precisa converter no cliente se for PDF/DOCX)
   * 
   * O Vectorizer automaticamente:
   * - Converte para Markdown (se usar /files/upload)
   * - Extrai texto do arquivo
   * - Divide em chunks
   * - Gera embeddings
   * - Armazena na coleção
   */
  async uploadDocument(options: DocumentUploadOptions): Promise<any> {
    const fileSize = fs.statSync(options.filePath).size;
    const maxMultipartSize = 2 * 1024 * 1024; // 2MB
    const extension = path.extname(options.filePath).toLowerCase();
    const needsTransmutation = ['.pdf', '.docx', '.xlsx', '.pptx', '.html', '.xml'].includes(extension);

    // Se arquivo é pequeno E precisa de transmutation, usar /files/upload
    if (fileSize <= maxMultipartSize && needsTransmutation) {
      return await this.uploadViaMultipart(options);
    }

    // Para arquivos grandes ou texto simples, usar /api/v1/insert
    return await this.uploadViaInsert(options);
  }

  /**
   * Upload via /files/upload (com transmutation automático)
   * Limite: 2MB
   */
  private async uploadViaMultipart(options: DocumentUploadOptions): Promise<any> {
    const formData = new FormData();
    formData.append('file', fs.createReadStream(options.filePath));
    formData.append('collection_name', options.collectionName);
    
    if (options.chunkSize) {
      formData.append('chunk_size', options.chunkSize.toString());
    }
    if (options.chunkOverlap) {
      formData.append('chunk_overlap', options.chunkOverlap.toString());
    }
    if (options.metadata) {
      formData.append('metadata', JSON.stringify(options.metadata));
    }
    
    const client = axios.create({
      baseURL: this.client.baseUrl,
      headers: {
        ...formData.getHeaders(),
        'X-HiveHub-User-ID': this.client.tenantId,
        'X-HiveHub-Service': this.client.serviceName,
      },
      maxContentLength: Infinity,
      maxBodyLength: Infinity,
    });
    
    const response = await client.post('/files/upload', formData);
    return response.data;
  }

  /**
   * Upload via /api/v1/insert (sem limite de tamanho)
   * ⚠️ Para PDF/DOCX, você precisa converter para Markdown no cliente primeiro
   */
  private async uploadViaInsert(options: DocumentUploadOptions): Promise<any> {
    
    const extension = path.extname(options.filePath).toLowerCase();
    const needsTransmutation = ['.pdf', '.docx', '.xlsx', '.pptx', '.html', '.xml'].includes(extension);
    
    let text: string;

    if (needsTransmutation) {
      // ⚠️ Você precisa de uma biblioteca de conversão aqui
      // Exemplo: usar uma API externa ou biblioteca transmutation para Node.js
      throw new Error(
        `Arquivo ${extension} precisa de conversão para Markdown. ` +
        `Use uma biblioteca de conversão (pdf-parse, mammoth, etc.) ou ` +
        `use /files/upload para arquivos < 2MB.`
      );
    } else {
      // Arquivo de texto - ler diretamente
      text = fs.readFileSync(options.filePath, 'utf-8');
    }

    // Dividir em chunks
    const chunks = this.chunkText(
      text,
      options.chunkSize || 1000,
      options.chunkOverlap || 200
    );

    // Enviar cada chunk
    const client = axios.create({
      baseURL: this.client.baseUrl,
      headers: {
        'Content-Type': 'application/json',
        'X-HiveHub-User-ID': this.client.tenantId,
        'X-HiveHub-Service': this.client.serviceName,
      },
    });

    const results = [];
    for (let i = 0; i < chunks.length; i++) {
      const result = await client.post('/api/v1/insert', {
        collection: options.collectionName,
        text: chunks[i],
        metadata: {
          ...options.metadata,
          file_path: options.filePath,
          chunk_index: i,
          total_chunks: chunks.length,
          original_filename: path.basename(options.filePath),
          file_extension: extension,
        },
      });
      results.push(result.data);
    }

    return {
      success: true,
      filename: path.basename(options.filePath),
      collection_name: options.collectionName,
      chunks_created: chunks.length,
      vectors_created: chunks.length,
    };
  }

  /**
   * Dividir texto em chunks
   */
  private chunkText(text: string, chunkSize: number, chunkOverlap: number): string[] {
    const chunks: string[] = [];
    let start = 0;

    while (start < text.length) {
      const end = Math.min(start + chunkSize, text.length);
      chunks.push(text.slice(start, end));
      start += chunkSize - chunkOverlap;
      if (start >= text.length) break;
    }

    return chunks;
  }

  /**
   * Processar múltiplos documentos
   */
  async uploadDocuments(
    files: Array<{ path: string; metadata?: Record<string, any> }>,
    collectionName: string
  ): Promise<any[]> {
    const results = [];
    
    for (const file of files) {
      try {
        const result = await this.uploadDocument({
          filePath: file.path,
          collectionName,
          metadata: file.metadata,
        });
        results.push({ file: file.path, success: true, result });
      } catch (error: any) {
        results.push({ file: file.path, success: false, error: error.message });
      }
    }
    
    return results;
  }

  /**
   * Processar texto direto (sem arquivo)
   * Usa /api/v1/insert diretamente (mais eficiente)
   */
  async processText(
    text: string,
    collectionName: string,
    metadata?: Record<string, any>
  ): Promise<any> {
    // Dividir em chunks
    const chunks = this.chunkText(text, 1000, 200);

    // Enviar cada chunk via /api/v1/insert
    const client = axios.create({
      baseURL: this.client.baseUrl,
      headers: {
        'Content-Type': 'application/json',
        'X-HiveHub-User-ID': this.client.tenantId,
        'X-HiveHub-Service': this.client.serviceName,
      },
    });

    const results = [];
    for (let i = 0; i < chunks.length; i++) {
      const result = await client.post('/api/v1/insert', {
        collection: collectionName,
        text: chunks[i],
        metadata: {
          ...metadata,
          chunk_index: i,
          total_chunks: chunks.length,
          source: 'text_direct',
        },
      });
      results.push(result.data);
    }

    return {
      success: true,
      chunks_created: chunks.length,
      vectors_created: chunks.length,
    };
  }
}

// Exemplo de uso
async function exemploProcessamento() {
  const client = new VectorizerClient({
    baseUrl: 'http://localhost:15002',
    tenantId: '550e8400-e29b-41d4-a716-446655440000',
    serviceName: 'meu-app',
  });

  const processor = new DocumentProcessor(client);

  // 1. Criar coleção primeiro
  await client.createCollection({
    name: 'documentos',
    dimension: 384, // Dimensão do embedding (ajuste conforme seu modelo)
    metric: 'cosine',
  });

  // 2. Processar um PDF (se < 2MB, usa transmutation automático)
  // Se > 2MB, você precisa converter para Markdown no cliente primeiro
  const resultado = await processor.uploadDocument({
    filePath: './documento.pdf',
    collectionName: 'documentos',
    chunkSize: 1000,      // 1000 caracteres por chunk
    chunkOverlap: 200,    // 200 caracteres de sobreposição
    metadata: {
      author: 'João Silva',
      category: 'técnico',
      date: '2025-01-15',
    },
  });

  // O PDF foi automaticamente:
  // 1. Convertido para Markdown (se < 2MB e usando /files/upload)
  // 2. Dividido por páginas (com marcadores --- Page X ---)
  // 3. Chunked em pedaços menores
  // 4. Convertido em vetores
  // 5. Armazenado na coleção com metadados (incluindo page_count, title, author, etc.)

  console.log('Documento processado:', {
    filename: resultado.filename,
    chunks: resultado.chunks_created,
    vectors: resultado.vectors_created,
    tempo: `${resultado.processing_time_ms}ms`,
  });

  // 3. Processar múltiplos arquivos
  // Arquivos < 2MB: transmutation automático via /files/upload
  // Arquivos > 2MB: precisa converter no cliente primeiro
  const resultados = await processor.uploadDocuments(
    [
      { path: './doc1.pdf', metadata: { source: 'cliente-a' } },    // ✅ PDF → Markdown (se < 2MB)
      { path: './doc2.docx', metadata: { source: 'cliente-b' } },  // ✅ DOCX → Markdown (se < 2MB)
      { path: './doc3.txt', metadata: { source: 'cliente-c' } },  // ✅ TXT (qualquer tamanho)
    ],
    'documentos'
  );

  console.log('Resultados:', resultados);

  // 4. Processar texto direto
  await processor.processText(
    'Este é um texto que será convertido em vetores automaticamente.',
    'documentos',
    { source: 'api', type: 'texto-direto' }
  );
}
```

### Golang - Upload de Documentos

```go
// document/processor.go
package document

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"mime/multipart"
	"net/http"
	"os"
	"path/filepath"
	"vectorizer-tenant-client/vectorizer"
)

// Processor processador de documentos
type Processor struct {
	client *vectorizer.Client
}

// NewProcessor cria um novo processador
func NewProcessor(client *vectorizer.Client) *Processor {
	return &Processor{
		client: client,
	}
}

// UploadOptions opções de upload
type UploadOptions struct {
	FilePath      string
	CollectionName string
	ChunkSize     int
	ChunkOverlap  int
	Metadata      map[string]interface{}
	PublicKey     string
}

// UploadResponse resposta do upload
type UploadResponse struct {
	Success          bool   `json:"success"`
	Filename         string `json:"filename"`
	CollectionName   string `json:"collection_name"`
	ChunksCreated    int    `json:"chunks_created"`
	VectorsCreated   int    `json:"vectors_created"`
	FileSize         int    `json:"file_size"`
	Language         string `json:"language"`
	ProcessingTimeMs int64  `json:"processing_time_ms"`
}

// UploadDocument processa e envia documento para coleção
func (p *Processor) UploadDocument(options UploadOptions) (*UploadResponse, error) {
	// Abrir arquivo
	file, err := os.Open(options.FilePath)
	if err != nil {
		return nil, fmt.Errorf("failed to open file: %w", err)
	}
	defer file.Close()

	// Criar multipart form
	var requestBody bytes.Buffer
	writer := multipart.NewWriter(&requestBody)

	// Adicionar arquivo
	part, err := writer.CreateFormFile("file", filepath.Base(options.FilePath))
	if err != nil {
		return nil, fmt.Errorf("failed to create form file: %w", err)
	}

	if _, err := io.Copy(part, file); err != nil {
		return nil, fmt.Errorf("failed to copy file: %w", err)
	}

	// Adicionar collection_name
	if err := writer.WriteField("collection_name", options.CollectionName); err != nil {
		return nil, fmt.Errorf("failed to write collection_name: %w", err)
	}

	// Adicionar configurações opcionais
	if options.ChunkSize > 0 {
		if err := writer.WriteField("chunk_size", fmt.Sprintf("%d", options.ChunkSize)); err != nil {
			return nil, fmt.Errorf("failed to write chunk_size: %w", err)
		}
	}

	if options.ChunkOverlap > 0 {
		if err := writer.WriteField("chunk_overlap", fmt.Sprintf("%d", options.ChunkOverlap)); err != nil {
			return nil, fmt.Errorf("failed to write chunk_overlap: %w", err)
		}
	}

	if options.Metadata != nil {
		metadataJSON, err := json.Marshal(options.Metadata)
		if err != nil {
			return nil, fmt.Errorf("failed to marshal metadata: %w", err)
		}
		if err := writer.WriteField("metadata", string(metadataJSON)); err != nil {
			return nil, fmt.Errorf("failed to write metadata: %w", err)
		}
	}

	if options.PublicKey != "" {
		if err := writer.WriteField("public_key", options.PublicKey); err != nil {
			return nil, fmt.Errorf("failed to write public_key: %w", err)
		}
	}

	if err := writer.Close(); err != nil {
		return nil, fmt.Errorf("failed to close writer: %w", err)
	}

	// Criar requisição
	req, err := http.NewRequest("POST", p.client.BaseURL()+"/files/upload", &requestBody)
	if err != nil {
		return nil, fmt.Errorf("failed to create request: %w", err)
	}

	req.Header.Set("Content-Type", writer.FormDataContentType())
	// Headers de isolamento por tenant (precisamos expor no client)
	// req.Header.Set("X-HiveHub-User-ID", p.client.TenantID())
	// req.Header.Set("X-HiveHub-Service", p.client.ServiceName())

	// Fazer requisição
	client := &http.Client{}
	resp, err := client.Do(req)
	if err != nil {
		return nil, fmt.Errorf("request failed: %w", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		body, _ := io.ReadAll(resp.Body)
		return nil, fmt.Errorf("upload failed: %s", string(body))
	}

	var result UploadResponse
	if err := json.NewDecoder(resp.Body).Decode(&result); err != nil {
		return nil, fmt.Errorf("failed to decode response: %w", err)
	}

	return &result, nil
}

// ProcessText processa texto direto (sem arquivo)
func (p *Processor) ProcessText(
	text string,
	collectionName string,
	metadata map[string]interface{},
) (*UploadResponse, error) {
	// Criar arquivo temporário
	tmpFile, err := os.CreateTemp("", "text-*.txt")
	if err != nil {
		return nil, fmt.Errorf("failed to create temp file: %w", err)
	}
	defer os.Remove(tmpFile.Name())
	defer tmpFile.Close()

	if _, err := tmpFile.WriteString(text); err != nil {
		return nil, fmt.Errorf("failed to write text: %w", err)
	}
	tmpFile.Close()

	return p.UploadDocument(UploadOptions{
		FilePath:      tmpFile.Name(),
		CollectionName: collectionName,
		Metadata:      metadata,
	})
}

// UploadDocuments processa múltiplos documentos
func (p *Processor) UploadDocuments(
	files []string,
	collectionName string,
) ([]UploadResponse, error) {
	var results []UploadResponse

	for _, filePath := range files {
		result, err := p.UploadDocument(UploadOptions{
			FilePath:      filePath,
			CollectionName: collectionName,
		})
		if err != nil {
			return nil, fmt.Errorf("failed to upload %s: %w", filePath, err)
		}
		results = append(results, *result)
	}

	return results, nil
}
```

**Nota:** No cliente Golang, você precisa expor os métodos `TenantID()` e `ServiceName()`:

```go
// vectorizer/client.go (adicionar métodos)

func (c *Client) TenantID() string {
    return c.tenantID
}

func (c *Client) ServiceName() string {
    return c.serviceName
}

func (c *Client) BaseURL() string {
    return c.baseURL
}
```

### Exemplo Completo - Node.js

```typescript
// exemplo-completo.ts
import { VectorizerClient } from './vectorizer-client';
import { DocumentProcessor } from './document-processor';

async function exemploCompleto() {
  // 1. Configurar cliente para tenant
  const client = new VectorizerClient({
    baseUrl: 'http://localhost:15002',
    tenantId: '550e8400-e29b-41d4-a716-446655440000',
    serviceName: 'meu-app',
  });

  const processor = new DocumentProcessor(client);

  // 2. Criar coleção
  console.log('Criando coleção...');
  await client.createCollection({
    name: 'documentos-empresa',
    dimension: 384, // Ajuste conforme seu modelo de embedding
    metric: 'cosine',
  });

  // 3. Processar documento PDF
  console.log('Processando PDF...');
  const resultadoPDF = await processor.uploadDocument({
    filePath: './relatorio.pdf',
    collectionName: 'documentos-empresa',
    chunkSize: 1000,
    chunkOverlap: 200,
    metadata: {
      tipo: 'relatorio',
      departamento: 'vendas',
      ano: 2025,
    },
  });

  console.log('PDF processado:', {
    arquivo: resultadoPDF.filename,
    chunks: resultadoPDF.chunks_created,
    vetores: resultadoPDF.vectors_created,
    tempo: `${resultadoPDF.processing_time_ms}ms`,
  });

  // 4. Processar documento Word
  console.log('Processando DOCX...');
  const resultadoDOCX = await processor.uploadDocument({
    filePath: './contrato.docx',
    collectionName: 'documentos-empresa',
    metadata: {
      tipo: 'contrato',
      cliente: 'Empresa ABC',
    },
  });

  // 5. Processar texto direto
  console.log('Processando texto...');
  await processor.processText(
    `
    Este é um documento importante sobre nossa estratégia de vendas.
    Precisamos focar em clientes corporativos e expandir para novos mercados.
    `,
    'documentos-empresa',
    {
      tipo: 'nota',
      autor: 'CEO',
      data: new Date().toISOString(),
    }
  );

  // 6. Buscar documentos processados
  console.log('Buscando documentos...');
  const resultados = await client.search('documentos-empresa', {
    query: 'estratégia de vendas',
    limit: 5,
  });

  console.log('Resultados da busca:', resultados);

  // 7. Verificar isolamento (outro tenant não vê essas coleções)
  const outroTenant = new VectorizerClient({
    baseUrl: 'http://localhost:15002',
    tenantId: '660e8400-e29b-41d4-a716-446655440001', // Tenant diferente
    serviceName: 'meu-app',
  });

  try {
    await outroTenant.getCollectionInfo('documentos-empresa');
    console.error('ERRO: Outro tenant conseguiu acessar!');
  } catch (error: any) {
    console.log('✅ Isolamento funcionando:', error.message);
  }
}

exemploCompleto().catch(console.error);
```

### Exemplo Completo - Golang

```go
// exemplo-completo.go
package main

import (
	"fmt"
	"log"
	"vectorizer-tenant-client/document"
	"vectorizer-tenant-client/vectorizer"
)

func main() {
	// 1. Configurar cliente para tenant
	client := vectorizer.NewClient(vectorizer.Config{
		BaseURL:     "http://localhost:15002",
		TenantID:    "550e8400-e29b-41d4-a716-446655440000",
		ServiceName: "meu-app",
	})

	processor := document.NewProcessor(client)

	// 2. Criar coleção
	fmt.Println("Criando coleção...")
	_, err := client.CreateCollection(vectorizer.CollectionConfig{
		Name:      "documentos-empresa",
		Dimension: 384, // Ajuste conforme seu modelo de embedding
		Metric:    "cosine",
	})
	if err != nil {
		log.Fatal(err)
	}

	// 3. Processar documento PDF
	fmt.Println("Processando PDF...")
	resultadoPDF, err := processor.UploadDocument(document.UploadOptions{
		FilePath:      "./relatorio.pdf",
		CollectionName: "documentos-empresa",
		ChunkSize:     1000,
		ChunkOverlap:  200,
		Metadata: map[string]interface{}{
			"tipo":        "relatorio",
			"departamento": "vendas",
			"ano":         2025,
		},
	})
	if err != nil {
		log.Fatal(err)
	}

	fmt.Printf("PDF processado: %s - %d chunks, %d vetores, %dms\n",
		resultadoPDF.Filename,
		resultadoPDF.ChunksCreated,
		resultadoPDF.VectorsCreated,
		resultadoPDF.ProcessingTimeMs,
	)

	// 4. Processar texto direto
	fmt.Println("Processando texto...")
	_, err = processor.ProcessText(
		`
		Este é um documento importante sobre nossa estratégia de vendas.
		Precisamos focar em clientes corporativos e expandir para novos mercados.
		`,
		"documentos-empresa",
		map[string]interface{}{
			"tipo": "nota",
			"autor": "CEO",
		},
	)
	if err != nil {
		log.Fatal(err)
	}

	// 5. Buscar documentos processados
	fmt.Println("Buscando documentos...")
	resultados, err := client.Search("documentos-empresa", vectorizer.SearchRequest{
		Query: "estratégia de vendas",
		Limit: 5,
	})
	if err != nil {
		log.Fatal(err)
	}

	fmt.Printf("Resultados: %+v\n", resultados)
}
```

### Formatos Suportados

#### Com Transmutation (Conversão Automática para Markdown) - `/files/upload` apenas

**⚠️ Limite: 2MB** - Para arquivos maiores, use `/api/v1/insert` com conversão manual no cliente.

- **PDF:** `.pdf` - Conversão completa com divisão por páginas
- **Word:** `.docx` - Preserva estrutura, tabelas, listas
- **Excel:** `.xlsx` - Converte planilhas para tabelas Markdown
- **PowerPoint:** `.pptx` - Slides convertidos para páginas
- **HTML/XML:** `.html`, `.htm`, `.xml` - Estrutura preservada
- **Imagens:** `.jpg`, `.jpeg`, `.png`, `.tiff`, `.tif`, `.bmp`, `.gif`, `.webp` - OCR para extrair texto

#### Formatos de Texto Direto (Sem Conversão) - `/api/v1/insert` ou `/files/upload`

- **Textos:** `.txt`, `.md`, `.rst`
- **Código:** `.rs`, `.py`, `.js`, `.ts`, `.go`, `.java`, etc.
- **Dados:** `.json`, `.yaml`, `.csv`

**Nota:** Formatos de texto podem ser processados via `/api/v1/insert` sem limite de tamanho, ou via `/files/upload` se < 2MB.

### Configuração de Chunking

```typescript
// Configurações recomendadas
const configs = {
  // Documentos técnicos (código, APIs)
  tecnico: {
    chunkSize: 500,
    chunkOverlap: 100,
  },
  
  // Documentos gerais (artigos, relatórios)
  geral: {
    chunkSize: 1000,
    chunkOverlap: 200,
  },
  
  // Documentos longos (livros, manuais)
  longo: {
    chunkSize: 2000,
    chunkOverlap: 400,
  },
};
```

### Metadados Úteis

```typescript
// Exemplos de metadados para facilitar busca
const metadata = {
  // Identificação
  source: 'api',
  author: 'João Silva',
  created_at: new Date().toISOString(),
  
  // Categorização
  category: 'técnico',
  department: 'engenharia',
  project: 'projeto-x',
  
  // Controle
  version: '1.0',
  status: 'ativo',
  tags: ['urgente', 'importante'],
  
  // Relacionamentos
  related_documents: ['doc-123', 'doc-456'],
  parent_document: 'doc-789',
};
```

---

## Boas Práticas

### 1. Sempre Use UUIDs para Tenant IDs

```typescript
// ✅ Bom
const tenantId = '550e8400-e29b-41d4-a716-446655440000';

// ❌ Evite
const tenantId = 'tenant-1'; // Pode causar conflitos
```

### 2. Valide Tenant ID Antes de Usar

```go
func validateTenantID(tenantID string) error {
    _, err := uuid.Parse(tenantID)
    if err != nil {
        return fmt.Errorf("invalid tenant ID format: %w", err)
    }
    return nil
}
```

### 3. Cache de Clientes Vectorizer

```typescript
class VectorizerClientCache {
  private clients: Map<string, VectorizerClient> = new Map();

  getClient(tenantId: string): VectorizerClient {
    if (!this.clients.has(tenantId)) {
      this.clients.set(tenantId, new VectorizerClient({
        baseUrl: 'http://localhost:15002',
        tenantId,
      }));
    }
    return this.clients.get(tenantId)!;
  }
}
```

### 4. Tratamento de Erros

```go
func handleVectorizerError(err error) error {
    if strings.Contains(err.Error(), "not found") {
        return ErrCollectionNotFound
    }
    if strings.Contains(err.Error(), "quota") {
        return ErrQuotaExceeded
    }
    return err
}
```

### 5. Logging com Tenant Context

```typescript
function logWithTenant(tenantId: string, message: string) {
  console.log(`[Tenant: ${tenantId}] ${message}`);
}
```

---

## Troubleshooting

### Problema: Coleções não estão isoladas

**Solução:** Verifique se os headers estão sendo enviados:

```typescript
// Verificar headers
console.log('Headers:', {
  'X-HiveHub-User-ID': client.tenantId,
  'X-HiveHub-Service': client.serviceName,
});
```

### Problema: Erro 404 ao acessar coleção

**Causa:** A coleção pertence a outro tenant ou não existe.

**Solução:** Verifique se o `tenant_id` está correto e se a coleção foi criada com esse tenant.

### Problema: Tenant ID inválido

**Solução:** Certifique-se de que o `tenant_id` é um UUID válido:

```go
import "github.com/google/uuid"

func isValidUUID(s string) bool {
    _, err := uuid.Parse(s)
    return err == nil
}
```

### Problema: Headers não estão sendo passados

**Solução:** Verifique se o cliente HTTP está configurado corretamente:

```typescript
// Axios
const client = axios.create({
  baseURL: 'http://localhost:15002',
  headers: {
    'X-HiveHub-User-ID': tenantId,
    'X-HiveHub-Service': 'my-app',
  },
});
```

---

## Conclusão

O Vectorizer já possui toda a infraestrutura necessária para isolamento por tenant. Você só precisa:

1. ✅ Enviar `X-HiveHub-User-ID` e `X-HiveHub-Service` nos headers
2. ✅ Usar UUIDs válidos para tenant IDs
3. ✅ Criar um cliente por tenant
4. ✅ O isolamento é automático em todas as operações

**Não é necessário alterar o código do Vectorizer!** Apenas use os headers corretos e o sistema isola automaticamente.

---

## Referências

- [Multi-Tenancy Guide](./MULTI_TENANCY.md) - Documentação oficial de multi-tenancy
- [REST API Documentation](../api/README.md) - Documentação completa da API REST
- [MCP Tools Documentation](../mcp/README.md) - Documentação das ferramentas MCP

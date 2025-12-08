---
title: TLS/SSL Configuration
module: configuration
id: tls-configuration
order: 5
description: Configuring TLS/SSL for secure HTTPS connections
tags: [configuration, tls, ssl, https, security, certificates]
---

# TLS/SSL Configuration

Complete guide to configuring TLS/SSL for secure HTTPS connections in Vectorizer.

## Overview

Vectorizer supports TLS/SSL encryption for all HTTP connections, including:
- HTTPS endpoints for REST API
- gRPC with TLS
- Mutual TLS (mTLS) for client certificate authentication

## Quick Start

### Enable HTTPS

1. Generate or obtain SSL certificates
2. Configure TLS in `config.yml`:

```yaml
tls:
  enabled: true
  cert_path: /path/to/cert.pem
  key_path: /path/to/key.pem
```

3. Start Vectorizer - it will now accept HTTPS connections on the configured port.

## Configuration Options

### YAML Configuration

```yaml
tls:
  # Enable/disable TLS (default: false)
  enabled: true

  # Path to PEM-encoded certificate file
  cert_path: /etc/vectorizer/certs/server.pem

  # Path to PEM-encoded private key file
  key_path: /etc/vectorizer/certs/server-key.pem

  # Mutual TLS (client certificate validation)
  mtls_enabled: false

  # Path to CA certificate for client validation (required if mtls_enabled)
  client_ca_path: /etc/vectorizer/certs/ca.pem

  # Cipher suite preset (default: Modern)
  # Options: Modern, Compatible, Custom
  cipher_suites: Modern

  # ALPN protocol negotiation (default: Both)
  # Options: Http1, Http2, Both, None, Custom
  alpn: Both
```

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `VECTORIZER_TLS_ENABLED` | Enable TLS | `false` |
| `VECTORIZER_TLS_CERT_PATH` | Certificate file path | - |
| `VECTORIZER_TLS_KEY_PATH` | Private key file path | - |
| `VECTORIZER_TLS_MTLS_ENABLED` | Enable mTLS | `false` |
| `VECTORIZER_TLS_CLIENT_CA_PATH` | Client CA certificate | - |

## Cipher Suite Presets

### Modern (Recommended)

TLS 1.3 only with the strongest ciphers:
- `TLS13_AES_256_GCM_SHA384`
- `TLS13_AES_128_GCM_SHA256`
- `TLS13_CHACHA20_POLY1305_SHA256`

```yaml
tls:
  cipher_suites: Modern
```

### Compatible

Supports TLS 1.2 and TLS 1.3 for broader client compatibility:
- All TLS 1.3 ciphers
- `TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384`
- `TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256`
- `TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256`
- `TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384`
- `TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256`
- `TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256`

```yaml
tls:
  cipher_suites: Compatible
```

### Custom

Define specific cipher suites:

```yaml
tls:
  cipher_suites:
    Custom:
      - TLS13_AES_256_GCM_SHA384
      - TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384
```

## ALPN Configuration

Application-Layer Protocol Negotiation (ALPN) determines which protocol to use.

### Options

| Value | Description |
|-------|-------------|
| `Http1` | HTTP/1.1 only |
| `Http2` | HTTP/2 only |
| `Both` | HTTP/2 preferred, fallback to HTTP/1.1 |
| `None` | Disable ALPN |
| `Custom` | Custom protocol list |

### Example

```yaml
tls:
  alpn: Http2  # HTTP/2 only
```

### Custom ALPN

```yaml
tls:
  alpn:
    Custom:
      - h2
      - http/1.1
      - grpc
```

## Mutual TLS (mTLS)

Mutual TLS requires clients to present a valid certificate signed by a trusted CA.

### Configuration

```yaml
tls:
  enabled: true
  cert_path: /etc/vectorizer/certs/server.pem
  key_path: /etc/vectorizer/certs/server-key.pem
  mtls_enabled: true
  client_ca_path: /etc/vectorizer/certs/client-ca.pem
```

### Client Certificate Requirements

Clients must provide:
1. A certificate signed by the CA specified in `client_ca_path`
2. The corresponding private key

### Example Client Request (curl)

```bash
curl --cert client.pem --key client-key.pem \
     --cacert server-ca.pem \
     https://localhost:15002/health
```

## Generating Certificates

### Self-Signed Certificates (Development)

```bash
# Generate CA
openssl genrsa -out ca-key.pem 4096
openssl req -new -x509 -days 365 -key ca-key.pem -out ca.pem \
    -subj "/CN=Vectorizer CA"

# Generate server certificate
openssl genrsa -out server-key.pem 4096
openssl req -new -key server-key.pem -out server.csr \
    -subj "/CN=localhost"
openssl x509 -req -days 365 -in server.csr -CA ca.pem -CAkey ca-key.pem \
    -CAcreateserial -out server.pem \
    -extfile <(echo "subjectAltName=DNS:localhost,IP:127.0.0.1")
```

### Let's Encrypt (Production)

For production, use Let's Encrypt or another trusted CA:

```bash
certbot certonly --standalone -d your-domain.com

# Configure paths
tls:
  enabled: true
  cert_path: /etc/letsencrypt/live/your-domain.com/fullchain.pem
  key_path: /etc/letsencrypt/live/your-domain.com/privkey.pem
```

## Docker Configuration

### With Docker

```yaml
# docker-compose.yml
services:
  vectorizer:
    image: hivellm/vectorizer:latest
    ports:
      - "15002:15002"
    volumes:
      - ./certs:/etc/vectorizer/certs:ro
      - ./config.yml:/etc/vectorizer/config.yml:ro
    environment:
      - VECTORIZER_TLS_ENABLED=true
      - VECTORIZER_TLS_CERT_PATH=/etc/vectorizer/certs/server.pem
      - VECTORIZER_TLS_KEY_PATH=/etc/vectorizer/certs/server-key.pem
```

## Kubernetes Configuration

### Using Secrets

```yaml
apiVersion: v1
kind: Secret
metadata:
  name: vectorizer-tls
type: kubernetes.io/tls
data:
  tls.crt: <base64-encoded-cert>
  tls.key: <base64-encoded-key>
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: vectorizer
spec:
  template:
    spec:
      containers:
      - name: vectorizer
        volumeMounts:
        - name: tls
          mountPath: /etc/vectorizer/certs
          readOnly: true
        env:
        - name: VECTORIZER_TLS_ENABLED
          value: "true"
        - name: VECTORIZER_TLS_CERT_PATH
          value: /etc/vectorizer/certs/tls.crt
        - name: VECTORIZER_TLS_KEY_PATH
          value: /etc/vectorizer/certs/tls.key
      volumes:
      - name: tls
        secret:
          secretName: vectorizer-tls
```

## Troubleshooting

### Certificate Errors

**Error: "Failed to load certificate"**
- Verify the certificate file exists and is readable
- Ensure the certificate is in PEM format
- Check file permissions

**Error: "Failed to load private key"**
- Verify the key file exists and is readable
- Ensure the key is in PEM format and matches the certificate
- Check that the key is not encrypted (or provide the passphrase)

### Connection Errors

**Error: "Certificate not trusted"**
- For self-signed certificates, add the CA to the client's trust store
- For mTLS, ensure the client certificate is signed by the configured CA

**Error: "Protocol version not supported"**
- Use `cipher_suites: Compatible` for older clients
- Ensure the client supports TLS 1.2 or higher

### Testing TLS

```bash
# Test TLS connection
openssl s_client -connect localhost:15002 -showcerts

# Test with specific TLS version
openssl s_client -connect localhost:15002 -tls1_3

# Test ALPN
openssl s_client -connect localhost:15002 -alpn h2,http/1.1
```

## Security Best Practices

1. **Use Modern cipher suites** in production
2. **Enable mTLS** for service-to-service communication
3. **Rotate certificates** regularly (automate with cert-manager in Kubernetes)
4. **Use separate certificates** for different environments
5. **Store private keys securely** with appropriate file permissions (600)
6. **Monitor certificate expiration** and set up alerts

## See Also

- [Server Configuration](./SERVER.md)
- [Authentication](../api/AUTHENTICATION.md)
- [Kubernetes Deployment](../deployment/KUBERNETES.md)

"""
Tests for UMICP transport
"""

import pytest
import sys
sys.path.insert(0, '..')

from client import VectorizerClient
from utils.transport import TransportProtocol, parse_connection_string


class TestConnectionStringParsing:
    """Test connection string parsing."""
    
    def test_parse_http_connection_string(self):
        """Test parsing HTTP connection strings."""
        protocol, config = parse_connection_string("http://localhost:15002", "test-key")
        assert protocol == TransportProtocol.HTTP
        assert config["base_url"] == "http://localhost:15002"
        assert config["api_key"] == "test-key"
    
    def test_parse_https_connection_string(self):
        """Test parsing HTTPS connection strings."""
        protocol, config = parse_connection_string("https://api.example.com", "test-key")
        assert protocol == TransportProtocol.HTTP
        assert config["base_url"] == "https://api.example.com"
    
    def test_parse_umicp_connection_string(self):
        """Test parsing UMICP connection strings."""
        protocol, config = parse_connection_string("umicp://localhost:15003", "test-key")
        assert protocol == TransportProtocol.UMICP
        assert config["host"] == "localhost"
        assert config["port"] == 15003
        assert config["api_key"] == "test-key"
    
    def test_parse_umicp_default_port(self):
        """Test UMICP default port."""
        protocol, config = parse_connection_string("umicp://localhost", "test-key")
        assert config["port"] == 15003
    
    def test_parse_invalid_protocol(self):
        """Test invalid protocol."""
        with pytest.raises(ValueError, match="Unsupported protocol"):
            parse_connection_string("ftp://localhost", "test-key")


class TestVectorizerClientUMICP:
    """Test VectorizerClient with UMICP."""
    
    def test_umicp_from_connection_string(self):
        """Test initializing client from UMICP connection string."""
        client = VectorizerClient(
            connection_string="umicp://localhost:15003",
            api_key="test-key"
        )
        assert client.get_protocol() == "umicp"
    
    def test_umicp_explicit_configuration(self):
        """Test initializing client with explicit UMICP configuration."""
        client = VectorizerClient(
            protocol="umicp",
            api_key="test-key",
            umicp={"host": "localhost", "port": 15003}
        )
        assert client.get_protocol() == "umicp"
    
    def test_http_by_default(self):
        """Test HTTP is used by default."""
        client = VectorizerClient(base_url="http://localhost:15002")
        assert client.get_protocol() == "http"
    
    def test_umicp_requires_config(self):
        """Test UMICP requires configuration."""
        with pytest.raises(ValueError, match="UMICP configuration is required"):
            VectorizerClient(protocol="umicp", api_key="test-key")
    
    def test_umicp_default_host_port(self):
        """Test UMICP with default host and port."""
        client = VectorizerClient(
            protocol="umicp",
            api_key="test-key",
            umicp={}
        )
        assert client.get_protocol() == "umicp"


class TestUMICPPerformance:
    """Test UMICP performance benefits."""
    
    def test_umicp_protocol_support(self):
        """Test UMICP protocol is supported."""
        client = VectorizerClient(
            protocol="umicp",
            api_key="test-key",
            umicp={"host": "localhost", "port": 15003}
        )
        assert client.get_protocol() == "umicp"
    
    def test_umicp_configuration_options(self):
        """Test UMICP supports custom configuration."""
        client = VectorizerClient(
            protocol="umicp",
            api_key="test-key",
            timeout=60,
            umicp={"host": "localhost", "port": 15003}
        )
        assert client.get_protocol() == "umicp"
        assert client.timeout == 60


if __name__ == "__main__":
    pytest.main([__file__, "-v"])


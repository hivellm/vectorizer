"""UMICP Tool Discovery - MCP-compatible interfaces.

This module provides interfaces for services that support automatic tool discovery,
compatible with the Model Context Protocol (MCP).
"""

from typing import Protocol, Dict, List, Optional, Any
from dataclasses import dataclass, field


@dataclass
class OperationSchema:
    """Schema for an operation/tool, compatible with MCP JSON Schema.

    Attributes:
        name: Operation name (e.g., "search_vectors")
        input_schema: JSON Schema for input parameters
        title: Display title (optional)
        description: Description of what the operation does (optional)
        output_schema: JSON Schema for output (optional)
        annotations: Metadata annotations (optional)
    """

    name: str
    input_schema: Dict[str, Any]
    title: Optional[str] = None
    description: Optional[str] = None
    output_schema: Optional[Dict[str, Any]] = None
    annotations: Optional[Dict[str, Any]] = None

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary representation.

        Returns:
            Dictionary with all fields, omitting None values
        """
        result = {
            "name": self.name,
            "input_schema": self.input_schema,
        }

        if self.title is not None:
            result["title"] = self.title
        if self.description is not None:
            result["description"] = self.description
        if self.output_schema is not None:
            result["output_schema"] = self.output_schema
        if self.annotations is not None:
            result["annotations"] = self.annotations

        return result


@dataclass
class ServerInfo:
    """Server information for discovery.

    Attributes:
        server: Server name/identifier
        version: Server version
        protocol: Protocol version (e.g., "UMICP/0.2")
        features: List of supported features (optional)
        operations_count: Number of available operations (optional)
        mcp_compatible: Whether server is MCP-compatible (optional)
        metadata: Additional metadata (optional)
    """

    server: str
    version: str
    protocol: str
    features: Optional[List[str]] = None
    operations_count: Optional[int] = None
    mcp_compatible: Optional[bool] = None
    metadata: Optional[Dict[str, Any]] = None

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary representation.

        Returns:
            Dictionary with all fields, omitting None values
        """
        result = {
            "server": self.server,
            "version": self.version,
            "protocol": self.protocol,
        }

        if self.features is not None:
            result["features"] = self.features
        if self.operations_count is not None:
            result["operations_count"] = self.operations_count
        if self.mcp_compatible is not None:
            result["mcp_compatible"] = self.mcp_compatible
        if self.metadata is not None:
            result["metadata"] = self.metadata

        return result


class DiscoverableService(Protocol):
    """Protocol for services that support tool discovery.

    Implement this protocol to provide automatic tool discovery
    and introspection for your UMICP service.

    Example:
        >>> class MyService:
        ...     def list_operations(self) -> List[OperationSchema]:
        ...         return [
        ...             OperationSchema(
        ...                 name="search",
        ...                 title="Search",
        ...                 description="Search for items",
        ...                 input_schema={
        ...                     "type": "object",
        ...                     "properties": {
        ...                         "query": {"type": "string"},
        ...                         "limit": {"type": "integer", "default": 10}
        ...                     },
        ...                     "required": ["query"]
        ...                 },
        ...                 annotations={"read_only": True}
        ...             )
        ...         ]
        ...
        ...     def server_info(self) -> ServerInfo:
        ...         return ServerInfo(
        ...             server="my-service",
        ...             version="1.0.0",
        ...             protocol="UMICP/0.2",
        ...             features=["discovery", "search"],
        ...             mcp_compatible=True
        ...         )
    """

    def list_operations(self) -> List[OperationSchema]:
        """List all available operations with their schemas.

        Returns:
            List of operation schemas describing all available operations
        """
        ...

    def get_schema(self, name: str) -> Optional[OperationSchema]:
        """Get schema for a specific operation by name.

        Default implementation searches through list_operations().
        Override for more efficient lookup if needed.

        Args:
            name: Operation name to look up

        Returns:
            Operation schema if found, None otherwise
        """
        for op in self.list_operations():
            if op.name == name:
                return op
        return None

    def server_info(self) -> ServerInfo:
        """Get server information and metadata.

        Returns:
            Server information including version, protocol, features
        """
        ...


def generate_operations_response(service: DiscoverableService) -> Dict[str, Any]:
    """Generate JSON response for _list_operations request.

    Helper function to create a complete response for the _list_operations
    discovery operation.

    Args:
        service: Service implementing DiscoverableService protocol

    Returns:
        Dictionary with operations array and metadata
    """
    operations = service.list_operations()
    info = service.server_info()

    return {
        "operations": [op.to_dict() for op in operations],
        "count": len(operations),
        "protocol": info.protocol,
        "mcp_compatible": info.mcp_compatible if info.mcp_compatible is not None else False,
    }


def generate_schema_response(service: DiscoverableService, operation_name: str) -> Dict[str, Any]:
    """Generate JSON response for _get_schema request.

    Args:
        service: Service implementing DiscoverableService protocol
        operation_name: Name of operation to get schema for

    Returns:
        Dictionary with schema or error if not found
    """
    schema = service.get_schema(operation_name)

    if schema:
        return schema.to_dict()
    else:
        return {
            "error": "Operation not found",
            "operation": operation_name
        }


def generate_server_info_response(service: DiscoverableService) -> Dict[str, Any]:
    """Generate JSON response for _server_info request.

    Args:
        service: Service implementing DiscoverableService protocol

    Returns:
        Dictionary with complete server information
    """
    return service.server_info().to_dict()


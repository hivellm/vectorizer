"""
UMICP Compression Module

Provides compression and decompression utilities for efficient data transmission.
Supports GZIP and DEFLATE algorithms.
"""

import gzip
import zlib
from enum import Enum
from typing import Optional


class CompressionType(Enum):
    """Compression algorithm types"""
    NONE = "none"
    GZIP = "gzip"
    DEFLATE = "deflate"
    LZ4 = "lz4"  # Placeholder for future implementation


class CompressionError(Exception):
    """Raised when compression/decompression fails"""
    pass


class Compression:
    """
    Compression utilities for UMICP

    Supports GZIP and DEFLATE compression algorithms for efficient
    data transmission over the network.
    """

    @staticmethod
    def compress(data: bytes, compression_type: CompressionType = CompressionType.GZIP) -> bytes:
        """
        Compress data using specified algorithm

        Args:
            data: Data to compress
            compression_type: Compression algorithm to use

        Returns:
            Compressed data as bytes

        Raises:
            CompressionError: If compression fails
            ValueError: If data or compression_type is invalid
        """
        if not isinstance(data, bytes):
            raise ValueError("Data must be bytes")

        if not isinstance(compression_type, CompressionType):
            raise ValueError("compression_type must be CompressionType enum")

        if compression_type == CompressionType.NONE:
            return data

        try:
            if compression_type == CompressionType.GZIP:
                return gzip.compress(data)
            elif compression_type == CompressionType.DEFLATE:
                return zlib.compress(data)
            elif compression_type == CompressionType.LZ4:
                raise CompressionError("LZ4 compression not yet implemented")
            else:
                raise CompressionError(f"Unsupported compression type: {compression_type}")
        except Exception as e:
            if isinstance(e, CompressionError):
                raise
            raise CompressionError(f"Compression failed: {str(e)}") from e

    @staticmethod
    def decompress(data: bytes, compression_type: CompressionType = CompressionType.GZIP) -> bytes:
        """
        Decompress data using specified algorithm

        Args:
            data: Compressed data
            compression_type: Compression algorithm used

        Returns:
            Decompressed data as bytes

        Raises:
            CompressionError: If decompression fails
            ValueError: If data or compression_type is invalid
        """
        if not isinstance(data, bytes):
            raise ValueError("Data must be bytes")

        if not isinstance(compression_type, CompressionType):
            raise ValueError("compression_type must be CompressionType enum")

        if compression_type == CompressionType.NONE:
            return data

        try:
            if compression_type == CompressionType.GZIP:
                return gzip.decompress(data)
            elif compression_type == CompressionType.DEFLATE:
                return zlib.decompress(data)
            elif compression_type == CompressionType.LZ4:
                raise CompressionError("LZ4 decompression not yet implemented")
            else:
                raise CompressionError(f"Unsupported compression type: {compression_type}")
        except Exception as e:
            if isinstance(e, CompressionError):
                raise
            raise CompressionError(f"Decompression failed: {str(e)}") from e

    @staticmethod
    def get_compression_ratio(original_size: int, compressed_size: int) -> float:
        """
        Calculate compression ratio

        Args:
            original_size: Original data size in bytes
            compressed_size: Compressed data size in bytes

        Returns:
            Compression ratio (e.g., 2.5 means 2.5x reduction)
        """
        if compressed_size == 0:
            return 0.0
        return original_size / compressed_size

    @staticmethod
    def is_beneficial(original_size: int, compressed_size: int, threshold: float = 0.9) -> bool:
        """
        Check if compression is beneficial

        Args:
            original_size: Original data size
            compressed_size: Compressed data size
            threshold: Threshold ratio (default 0.9 = 10% savings minimum)

        Returns:
            True if compressed size is smaller than threshold * original size
        """
        return compressed_size < (original_size * threshold)


__all__ = ['Compression', 'CompressionType', 'CompressionError']


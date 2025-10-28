"""UMICP Matrix operations implementation."""

from typing import List, Union, Optional
from dataclasses import dataclass
import numpy as np

from umicp.error import MatrixOperationError


@dataclass
class MatrixResult:
    """Matrix operation result."""

    result: np.ndarray
    rows: int
    cols: int
    computation_time_ms: float = 0.0

    def to_list(self) -> List[List[float]]:
        """Convert to nested list."""
        return self.result.tolist()


@dataclass
class DotProductResult:
    """Dot product result."""

    result: float
    computation_time_ms: float = 0.0


@dataclass
class CosineSimilarityResult:
    """Cosine similarity result."""

    similarity: float
    computation_time_ms: float = 0.0


class Matrix:
    """High-performance matrix operations using NumPy."""

    def __init__(self) -> None:
        """Initialize matrix operations."""
        pass

    def add(self, a: np.ndarray, b: np.ndarray) -> MatrixResult:
        """Matrix addition.

        Args:
            a: First matrix
            b: Second matrix

        Returns:
            MatrixResult with sum

        Raises:
            MatrixOperationError: If matrices have incompatible shapes
        """
        try:
            if a.shape != b.shape:
                raise MatrixOperationError(
                    f"Incompatible shapes: {a.shape} and {b.shape}"
                )

            result = np.add(a, b)
            return MatrixResult(
                result=result,
                rows=result.shape[0],
                cols=result.shape[1] if result.ndim > 1 else 1,
            )
        except Exception as e:
            raise MatrixOperationError(f"Matrix addition failed: {e}")

    def multiply(self, a: np.ndarray, b: np.ndarray) -> MatrixResult:
        """Matrix multiplication.

        Args:
            a: First matrix
            b: Second matrix

        Returns:
            MatrixResult with product

        Raises:
            MatrixOperationError: If matrices have incompatible shapes
        """
        try:
            result = np.matmul(a, b)
            return MatrixResult(
                result=result,
                rows=result.shape[0],
                cols=result.shape[1] if result.ndim > 1 else 1,
            )
        except Exception as e:
            raise MatrixOperationError(f"Matrix multiplication failed: {e}")

    def transpose(self, matrix: np.ndarray) -> MatrixResult:
        """Matrix transpose.

        Args:
            matrix: Input matrix

        Returns:
            MatrixResult with transposed matrix
        """
        result = np.transpose(matrix)
        return MatrixResult(
            result=result,
            rows=result.shape[0],
            cols=result.shape[1] if result.ndim > 1 else 1,
        )

    def dot_product(self, a: np.ndarray, b: np.ndarray) -> DotProductResult:
        """Vector dot product.

        Args:
            a: First vector
            b: Second vector

        Returns:
            DotProductResult with dot product

        Raises:
            MatrixOperationError: If vectors have incompatible shapes
        """
        try:
            if a.shape != b.shape:
                raise MatrixOperationError(
                    f"Incompatible shapes: {a.shape} and {b.shape}"
                )

            result = float(np.dot(a, b))
            return DotProductResult(result=result)
        except Exception as e:
            raise MatrixOperationError(f"Dot product failed: {e}")

    def cosine_similarity(
        self, a: np.ndarray, b: np.ndarray
    ) -> CosineSimilarityResult:
        """Cosine similarity between vectors.

        Args:
            a: First vector
            b: Second vector

        Returns:
            CosineSimilarityResult with similarity score

        Raises:
            MatrixOperationError: If vectors have incompatible shapes
        """
        try:
            if a.shape != b.shape:
                raise MatrixOperationError(
                    f"Incompatible shapes: {a.shape} and {b.shape}"
                )

            dot_prod = np.dot(a, b)
            norm_a = np.linalg.norm(a)
            norm_b = np.linalg.norm(b)

            if norm_a == 0 or norm_b == 0:
                raise MatrixOperationError("Cannot compute similarity with zero vector")

            similarity = float(dot_prod / (norm_a * norm_b))
            return CosineSimilarityResult(similarity=similarity)
        except Exception as e:
            raise MatrixOperationError(f"Cosine similarity failed: {e}")

    def normalize(self, vector: np.ndarray) -> np.ndarray:
        """L2 normalization of vector.

        Args:
            vector: Input vector

        Returns:
            Normalized vector

        Raises:
            MatrixOperationError: If normalization fails
        """
        try:
            norm = np.linalg.norm(vector)
            if norm == 0:
                raise MatrixOperationError("Cannot normalize zero vector")
            return vector / norm
        except Exception as e:
            raise MatrixOperationError(f"Normalization failed: {e}")

    def vector_add(self, a: np.ndarray, b: np.ndarray) -> np.ndarray:
        """Vector addition.

        Args:
            a: First vector
            b: Second vector

        Returns:
            Sum vector
        """
        return np.add(a, b)

    def vector_subtract(self, a: np.ndarray, b: np.ndarray) -> np.ndarray:
        """Vector subtraction.

        Args:
            a: First vector
            b: Second vector

        Returns:
            Difference vector
        """
        return np.subtract(a, b)

    def vector_scale(self, vector: np.ndarray, scalar: float) -> np.ndarray:
        """Vector scaling.

        Args:
            vector: Input vector
            scalar: Scaling factor

        Returns:
            Scaled vector
        """
        return vector * scalar

    def determinant(self, matrix: np.ndarray) -> float:
        """Matrix determinant.

        Args:
            matrix: Square matrix

        Returns:
            Determinant value

        Raises:
            MatrixOperationError: If matrix is not square
        """
        try:
            if matrix.shape[0] != matrix.shape[1]:
                raise MatrixOperationError("Matrix must be square")
            return float(np.linalg.det(matrix))
        except Exception as e:
            raise MatrixOperationError(f"Determinant computation failed: {e}")

    def inverse(self, matrix: np.ndarray) -> MatrixResult:
        """Matrix inverse.

        Args:
            matrix: Square invertible matrix

        Returns:
            MatrixResult with inverse

        Raises:
            MatrixOperationError: If matrix is not invertible
        """
        try:
            result = np.linalg.inv(matrix)
            return MatrixResult(
                result=result,
                rows=result.shape[0],
                cols=result.shape[1],
            )
        except Exception as e:
            raise MatrixOperationError(f"Matrix inversion failed: {e}")


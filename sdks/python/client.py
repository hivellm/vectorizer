"""Backward-compatibility shim.

The Vectorizer Python SDK used to live in this single 2,907-line
module. It was split across the :mod:`vectorizer` package per API
surface in ``phase4_split-sdk-python-client``. Existing imports like
``from client import VectorizerClient`` keep working because this shim
re-exports the facade from its new home.

Prefer the new import going forward::

    from vectorizer import VectorizerClient
    from vectorizer.collections import CollectionsClient  # etc.
"""

from vectorizer.client import VectorizerClient

__all__ = ["VectorizerClient"]

"""
Setup configuration for Hive Vectorizer Python SDK.

This module contains the setup configuration for packaging
and distributing the Python SDK.
"""

from setuptools import setup, find_packages
import os

# Read README file
def read_readme():
    readme_path = os.path.join(os.path.dirname(__file__), "README.md")
    if os.path.exists(readme_path):
        with open(readme_path, "r", encoding="utf-8") as f:
            return f.read()
    return "Hive Vectorizer Python SDK"

# Read requirements
def read_requirements():
    requirements_path = os.path.join(os.path.dirname(__file__), "requirements.txt")
    if os.path.exists(requirements_path):
        with open(requirements_path, "r", encoding="utf-8") as f:
            return [line.strip() for line in f if line.strip() and not line.startswith("#")]
    return []

# Read version from __init__.py
def read_version():
    init_path = os.path.join(os.path.dirname(__file__), "vectorizer", "__init__.py")
    if os.path.exists(init_path):
        with open(init_path, "r", encoding="utf-8") as f:
            for line in f:
                if line.startswith("__version__"):
                    return line.split("=")[1].strip().strip('"').strip("'")
    return "1.0.0"

setup(
    name="hive-vectorizer",
    version=read_version(),
    author="CMMV-Hive Team",
    author_email="team@cmmv-hive.org",
    description="Python SDK for Hive Vectorizer - Semantic search and vector operations",
    long_description=read_readme(),
    long_description_content_type="text/markdown",
    url="https://github.com/cmmv-hive/vectorizer",
    project_urls={
        "Bug Reports": "https://github.com/cmmv-hive/vectorizer/issues",
        "Documentation": "https://docs.cmmv-hive.org/vectorizer",
        "Source": "https://github.com/cmmv-hive/vectorizer",
    },
    packages=find_packages(),
    classifiers=[
        "Development Status :: 5 - Production/Stable",
        "Intended Audience :: Developers",
        "License :: OSI Approved :: MIT License",
        "Operating System :: OS Independent",
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.8",
        "Programming Language :: Python :: 3.9",
        "Programming Language :: Python :: 3.10",
        "Programming Language :: Python :: 3.11",
        "Programming Language :: Python :: 3.12",
        "Topic :: Software Development :: Libraries :: Python Modules",
        "Topic :: Scientific/Engineering :: Artificial Intelligence",
        "Topic :: Text Processing :: Indexing",
        "Topic :: Internet :: WWW/HTTP :: Dynamic Content",
    ],
    python_requires=">=3.8",
    install_requires=read_requirements(),
    extras_require={
        "dev": [
            "pytest>=7.0.0",
            "pytest-asyncio>=0.21.0",
            "pytest-cov>=4.0.0",
            "black>=23.0.0",
            "isort>=5.12.0",
            "flake8>=6.0.0",
            "mypy>=1.0.0",
            "pre-commit>=3.0.0",
        ],
        "docs": [
            "sphinx>=6.0.0",
            "sphinx-rtd-theme>=1.2.0",
            "myst-parser>=1.0.0",
        ],
        "test": [
            "pytest>=7.0.0",
            "pytest-asyncio>=0.21.0",
            "pytest-cov>=4.0.0",
            "httpx>=0.24.0",
        ],
    },
    entry_points={
        "console_scripts": [
            "vectorizer-cli=vectorizer.cli:main",
        ],
    },
    include_package_data=True,
    package_data={
        "vectorizer": ["py.typed"],
    },
    zip_safe=False,
    keywords=[
        "vectorizer",
        "semantic-search",
        "embeddings",
        "machine-learning",
        "ai",
        "search",
        "vectors",
        "similarity",
        "hive",
        "cmmv",
    ],
)

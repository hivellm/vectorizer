# 📁 File Watcher System

## 🎯 Overview

The File Watcher System is a real-time file monitoring component of the Vectorizer that automatically tracks changes in indexed files and updates the vector database without requiring manual intervention or system restarts.

## ✨ Key Features

- **Real-time Monitoring**: Instant detection of file changes
- **Automatic Reindexing**: Seamless vector database updates
- **Pattern-based Filtering**: Flexible include/exclude patterns
- **Performance Optimized**: Debounced events and efficient processing
- **Cross-platform**: Works on Linux, macOS, and Windows
- **Robust Error Handling**: Graceful degradation and recovery

## 🚀 Quick Start

### **1. Configuration**

Create or update your `vectorize-workspace.yml`:

```yaml
global_settings:
  file_watcher:
    watch_paths:
      - "/path/to/your/project"
    auto_discovery: true
    enable_auto_update: true
    hot_reload: true
    exclude_patterns:
      - "**/.git/**"
      - "**/target/**"
      - "**/node_modules/**"
      - "**/*.log"
```

### **2. Start Vectorizer**

```bash
cd vectorizer
cargo run --release
```

The file watcher will automatically start and begin monitoring your configured paths.

## 📊 What It Does

### **Automatic Operations**

1. **File Creation**: New files are automatically indexed
2. **File Modification**: Changed files are reindexed
3. **File Deletion**: Removed files are cleaned from the database
4. **File Renaming**: Renamed files are properly tracked

### **Smart Filtering**

- **Include Patterns**: Only monitor files matching your patterns
- **Exclude Patterns**: Ignore system files and build artifacts
- **Content Validation**: Skip binary files and invalid content

## 🔧 Configuration Options

### **Global Settings**

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `watch_paths` | `Vec<String>` | `[]` | Directories to monitor |
| `auto_discovery` | `bool` | `true` | Enable automatic file discovery |
| `enable_auto_update` | `bool` | `true` | Enable automatic reindexing |
| `hot_reload` | `bool` | `true` | Enable hot reloading |
| `exclude_patterns` | `Vec<String>` | `["**/.git/**", "**/target/**"]` | Files to ignore |

### **Pattern Examples**

```yaml
include_patterns:
  - "**/*.md"          # All markdown files
  - "docs/**/*.rs"     # Rust files in docs directory
  - "src/**/*.{rs,py}" # Rust and Python files in src

exclude_patterns:
  - "**/.git/**"       # Git directories
  - "**/target/**"     # Build artifacts
  - "**/*.log"         # Log files
  - "**/node_modules/**" # Node.js dependencies
```

## 🏗️ Architecture

### **Core Components**

```
FileWatcherSystem
├── Watcher (File System Monitoring)
├── Debouncer (Event Aggregation)
├── VectorOperations (Database Updates)
├── FileWatcherConfig (Configuration)
└── MetricsCollector (Performance Tracking)
```

### **Event Flow**

```
File Change → Event Detection → Debouncing → Processing → Vector Update
```

## 📈 Performance

### **Optimizations**

- **Debounced Events**: Prevents excessive processing
- **Pattern Pre-filtering**: Reduces unnecessary file checks
- **Silent Filtering**: Minimizes log spam
- **Batch Processing**: Efficient database updates

### **Resource Usage**

- **CPU**: Minimal overhead (~1-2% on typical workloads)
- **Memory**: Efficient with large file sets
- **I/O**: Optimized file reading and writing

## 🛡️ Error Handling

### **Automatic Recovery**

- **Transient Errors**: Automatic retry with exponential backoff
- **Configuration Errors**: Graceful fallback to defaults
- **File System Errors**: Continue processing other files
- **Database Errors**: Log and continue operation

### **Monitoring**

- **Health Checks**: Built-in system health monitoring
- **Metrics Collection**: Performance and error tracking
- **Logging**: Comprehensive debug and error logging

## 🔍 Troubleshooting

### **Common Issues**

#### **File Watcher Not Starting**
```bash
# Check configuration
cat vectorize-workspace.yml

# Check logs
tail -f vectorizer.log
```

#### **Files Not Being Detected**
- Verify `watch_paths` configuration
- Check `include_patterns` match your files
- Ensure files aren't excluded by `exclude_patterns`

#### **Performance Issues**
- Review `exclude_patterns` to filter unnecessary files
- Check for large directories being monitored
- Monitor system resources

### **Debug Mode**

Enable detailed logging:

```bash
RUST_LOG=debug cargo run --release
```

## 📚 Documentation

### **Complete Documentation**

- **[Fixes Documentation](FILE_WATCHER_FIXES_DOCUMENTATION.md)**: All implemented fixes and solutions
- **[Architecture Documentation](FILE_WATCHER_ARCHITECTURE.md)**: System design and components
- **[API Documentation](FILE_WATCHER_API.md)**: Complete API reference
- **[Documentation Index](docs/FILE_WATCHER_DOCUMENTATION_INDEX.md)**: All available documents

### **Key Fixes Implemented**

1. **Self-Detection Loop Prevention**: Fixed infinite loops caused by Access events
2. **Empty Path Filtering**: Proper handling of ignored events
3. **Silent Filtering**: Reduced log spam for excluded files
4. **Quantization Fixes**: Prevented crashes with small vector sets
5. **Auto-save Optimization**: Single background save task

## 🧪 Testing

### **Test Coverage**

- **Unit Tests**: Individual component testing
- **Integration Tests**: End-to-end event processing
- **Load Tests**: High-frequency change scenarios
- **Error Tests**: Failure mode validation

### **Running Tests**

```bash
# Run all file watcher tests
cargo test file_watcher

# Run specific test categories
cargo test file_watcher::operations
cargo test file_watcher::debouncer
```

## 🚀 Production Deployment

### **Requirements**

- **Rust 1.70+**: For optimal performance
- **Memory**: 512MB+ recommended
- **Storage**: SSD recommended for better I/O performance

### **Best Practices**

1. **Configure Exclusions**: Exclude build artifacts and temporary files
2. **Monitor Resources**: Watch CPU and memory usage
3. **Log Management**: Implement log rotation
4. **Backup Strategy**: Regular vector database backups

## 🔮 Future Enhancements

### **Planned Features**

- **Distributed Monitoring**: Multi-node file watching
- **Advanced Analytics**: Usage pattern analysis
- **Performance Tuning**: Dynamic optimization
- **Web Interface**: Real-time monitoring dashboard

## 🤝 Contributing

### **Development Setup**

```bash
git clone <repository>
cd vectorizer
cargo build
cargo test
```

### **Code Style**

- Follow Rust conventions
- Add comprehensive tests
- Update documentation
- Include error handling

## 📄 License

This project is part of the Vectorizer system. See the main project license for details.

---

**Last Updated**: October 10, 2025  
**Version**: 1.0  
**Status**: ✅ Production Ready

## 🆘 Support

For issues, questions, or contributions:

1. Check the [troubleshooting section](#-troubleshooting)
2. Review the [complete documentation](#-documentation)
3. Open an issue with detailed information
4. Include logs and configuration details

# watchx 👀

[![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20macOS%20%7C%20Windows-blue.svg)](https://github.com/prongbang/watchx)
[![Homebrew](https://img.shields.io/badge/Homebrew-available-green.svg)](https://brew.sh)
[![Crates.io](https://img.shields.io/crates/v/watchx.svg)](https://crates.io/crates/watchx)
[![License](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

> Lightweight Live Reload Tool for Running Anything. Watch files and automatically restart your applications during development.

## ✨ Features

- 🚀 **Instant Reload** - Automatically restart your application on file changes
- ⚡ **Lightweight** - Minimal resource usage with efficient file watching
- 🔧 **Flexible Configuration** - Easy YAML configuration with environment variables
- 🎯 **Multiple Commands** - Run multiple commands simultaneously
- 🔍 **Smart Ignoring** - Powerful glob and regex patterns for ignoring files
- 🛠️ **Cross-Platform** - Works on Linux, macOS, and Windows

## 🚀 Quick Start

1. Create a `watchx.yaml` file in your project:

```shell
watchx init
```

This creates the default config:

```yaml
env:
  PORT: "8080"
commands:
  - "go run main.go"
watch_dir: "./"
ignore:
  - "**/.git/**"
  - "**/target/**"
  - "**/node_modules/**"
  - "*.log"
  - "*.tmp"
```

2. Run watchx:

```shell
watchx run
```

That's it! Your application will now automatically reload on file changes.

## 📦 Installation

### Via Homebrew (macOS & Linux)

```shell
brew install prongbang/homebrew-formulae/watchx
```

### Via Cargo

```shell
cargo install watchx --git https://github.com/prongbang/watchx.git
```

### From Source

```shell
# Install Rust if you haven't already
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and install
git clone https://github.com/prongbang/watchx.git
cd watchx
cargo install --path .
```

## ⚙️ Configuration

### Basic Configuration

Create a `watchx.yaml` file:

```yaml
# Environment variables
env:
  PORT: "8080"
  ENV: "development"

# Commands to run
commands:
  - "npm run dev"
  - "go run main.go"

# Directory to watch
watch_dir: "./"

# Files and directories to ignore
ignore:
  - "**/node_modules/**"
  - "**/.git/**"
```

Each command is started as an executable plus whitespace-separated arguments. Shell syntax such as pipes, redirects, inline environment assignments, and quoted arguments with spaces are not interpreted; put environment variables in `env`.

## 🔍 Ignore Patterns

watchx supports two types of patterns: **glob patterns** and **regex patterns**.

### Glob Patterns

Simple shell-style wildcards:

```yaml
ignore:
  # Match files by extension
  - "*.log"              # All .log files
  - "*.tmp"              # All .tmp files
  
  # Match directories
  - "**/target/**"       # target directory and contents
  - "**/node_modules/**" # node_modules and contents
  - "dist/"             # dist directory only
  
  # Match specific paths
  - "build/output/*.js"  # .js files in build/output
  - "test/**/*.test.js" # All test files
```

Special characters:
- `*` matches any number of characters except `/`
- `**` matches zero or more directories
- `?` matches any single character
- `[abc]` matches any character inside the brackets
- `/` at the end matches only directories

### Regex Patterns

Complex patterns enclosed in forward slashes:

```yaml
ignore:
  # File patterns
  - "/(^|/)test_.*\\.rs$/" # Files starting with test_ and ending with .rs
  - "/(^|/).*_test\\.go$/" # Files ending with _test.go
  
  # Directories
  - "/\\.git/"           # .git directory
  - "/build-\\d+/"       # build-{number} directories
  
  # Complex patterns
  - "/\\.(jpg|jpeg|png)$/" # Image files
  - "/^(dev|stage)_/"    # Files starting with dev_ or stage_
```

### Common Patterns

Here's a comprehensive example:

```yaml
ignore:
  # Development directories
  - "**/node_modules/**"
  - "**/target/**"
  - "**/dist/**"
  - "**/build/**"
  
  # Version control
  - "/\\.git/"
  - "/\\.svn/"
  
  # Build artifacts
  - "*.o"
  - "*.pyc"
  - "*.class"
  
  # Temporary files
  - "*.tmp"
  - "*.temp"
  - "*~"
  - "*.swp"
  
  # Logs and databases
  - "*.log"
  - "*.sqlite"
  
  # IDE files
  - "/\\.idea/"
  - "/\\.vscode/"
  
  # Test files
  - "/test_.*/"
  - "**/*_test.go"
  - "**/*.spec.js"
```

## 💻 Command Line Usage

### Basic Commands

```shell
# Run with default config
watchx run

# Initialize default config
watchx init

# Initialize custom config
watchx init -c custom.yaml

# Overwrite an existing config
watchx init -f

# Run with custom config
watchx run -c custom.yaml

# Show help
watchx --help

# Show version
watchx --version
```

### Options

| Command | Option | Short | Description |
|---------|--------|-------|-------------|
| `watchx init` | `--config <CONFIG>` | `-c` | Write the config file at a custom path |
| `watchx init` | `--force` | `-f` | Overwrite an existing config file |
| `watchx run` | `--config <CONFIG>` | `-c` | Read the config file from a custom path |
| all commands | `--help` | `-h` | Show help information |
| `watchx` | `--version` | `-V` | Show version information |

## 🔧 Use Cases

- **Go Development**: Automatically restart your Go server on code changes
- **Node.js Development**: Watch and reload your Node.js applications
- **Rust Development**: Recompile and run your Rust projects
- **Python Development**: Restart Python scripts automatically
- **Full-Stack Development**: Run frontend and backend simultaneously
- **Testing**: Auto-run tests on file changes

## 🤝 Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 💖 Support the Project

If you find this package helpful, please consider supporting it:

[!["Buy Me A Coffee"](https://www.buymeacoffee.com/assets/img/custom_images/orange_img.png)](https://www.buymeacoffee.com/prongbang)

## 🙏 Acknowledgments

- Built with Rust 🦀
- IDE Support by [RustRover](https://www.jetbrains.com/rust/)

![RustRover](https://resources.jetbrains.com/help/img/idea/2024.3/RustRover_icon.svg)

---

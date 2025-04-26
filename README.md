# watchx

Lightweight Live Reload Tool for Running Anything.

# How to use

## Create `watchx.yaml` file

```yaml
env:
  PORT: "8080"
commands:
  - "go run main.go"
watch_dir: "./"
ignore:
  - "**/.git/**"
  - "**/target/**"
  - "**/dist/**"
  - "**/build/**"
  - "**/node_modules/**"
  - "/.idea/"
  - "/.vscode/"
  - "*.log"
  - "*.tmp"
  - "*.temp"
  - "*~"
  - "*.swp"
```

## Run

- Uses default watchx.yaml

```shell
watchx run
```

- Uses custom config file

```shell
watchx run -c custom.yaml
```

# Install

### Install with Homebrew

```shell
brew update
brew tap prongbang/homebrew-formulae
brew install watchx
```

### Install with Rust

```shell
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

#### With Cargo Install

```shell
cargo install watchx --git https://github.com/prongbang/watchx.git
```

## Ignore Patterns

The `ignore` configuration supports two types of patterns: glob patterns and regex patterns.

### Glob Patterns

Glob patterns are simple pattern matching that use shell-style wildcards:

```yaml
ignore:
  # Match specific files
  - "*.log"              # Match all .log files
  - "*.tmp"              # Match all .tmp files
  
  # Match directories and their contents
  - "**/target/**"       # Match target directory and all its contents
  - "**/node_modules/**" # Match node_modules directory and all its contents
  - "dist/"             # Match dist directory
  
  # Match specific paths
  - "build/output/*.js"  # Match .js files in build/output
  - "test/**/*.test.js" # Match all test files
```

Special characters:

- * matches any number of characters except /
- ** matches zero or more directories
- ? matches any single character
- [abc] matches any character inside the brackets
- / at the end matches only directories

### Regex Patterns

For more complex matching, you can use regex patterns enclosed in forward slashes:

```yaml
ignore:
  # Match specific file patterns
  - "/^test_.*\\.rs$/"   # Match files starting with test_ and ending with .rs
  - "/.*_test\\.go$/"    # Match files ending with _test.go
  
  # Match directories
  - "/\\.git/"           # Match .git directory
  - "/build-\\d+/"      # Match build-{number} directories
  
  # Match complex patterns
  - "/\\.(jpg|jpeg|png)$/" # Match image files
  - "/^(dev|stage)_/"    # Match files starting with dev_ or stage_
 ```

### Examples

Common ignore patterns:

```yaml
ignore:
  # Development directories
  - "**/node_modules/**"
  - "**/target/**"
  - "**/dist/**"
  - "**/build/**"
  
  # Version control
  - "/\\.git/"
  - "/.svn/"
  
  # Build artifacts
  - "*.o"
  - "*.pyc"
  - "*.class"
  
  # Temporary files
  - "*.tmp"
  - "*.temp"
  - "*~"
  
  # Logs and databases
  - "*.log"
  - "*.sqlite"
  
  # IDE and editor files
  - "/.idea/"
  - "/.vscode/"
  - "*.swp"
  
  # Test files
  - "/test_.*/"
  - "/**/*_test.go"
  - "/**/*.spec.js"
 ```

## IDE

### Thank you [RustRover](https://www.jetbrains.com/rust/)

![RustRover](https://resources.jetbrains.com/help/img/idea/2024.3/RustRover_icon.svg)
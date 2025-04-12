# watchx

Lightweight Live Reload Tool for Running Anything.

# How to use

## Create `watchx.yaml` file

```yaml
env:
  PORT: "9001"
commands:
  - "make swaggen"
  - "go run cmd/api/main.go"
watch_dir: "./"
ignore:
  - ".git/"
  - "target/"
  - "docs/"
  - "node_modules/"
  - "*.log"
  - "*_test.go"
  - "*.json"
  - "*.yaml"
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

# reloadx

Lightweight Live Reload Tool for Running Anything.

# How to use

## Create `reloadx.yaml` file

```yaml
env:
  PORT: "8080"
commands:
  - "go run main.go"
watch_dir: "./"
ignore:
  - ".git"
  - "target"
  - "node_modules"
  - "*.log"
```

## Run

```shell
reloadx
```

# Install

### Install with Homebrew

```shell
brew update
brew tap prongbang/homebrew-formulae
brew install reloadx
```

or

### Install with Rust

```shell
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

#### With Cargo Install

```shell
cargo install reloadx --git https://github.com/prongbang/reloadx.git
```

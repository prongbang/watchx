# reloadx

Lightweight Live Reload Tool for Running Anything.

# Install

```shell
cargo install --path .
```

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
# reloadx

Lightweight Live Reload Tool for Running Anything.

# How to use

## Create `reloadx.yaml` file

```yaml
env:
  PORT: "8080"
  DEBUG: "true"
commands:
  - "go run main.go"
watch_dir: "./"
```

## Run

```shell
reloadx
```
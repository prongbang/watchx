# # Build for specific platform
# make build_windows
# make build_linux
# make build_macos

# # Build with specific version
# make build_windows VERSION=0.3.2
# make build_linux VERSION=0.3.2
# make build_macos VERSION=0.3.2

# # Build everything
# make build_all

# # Release everything with version
# make release_all VERSION=0.3.2

# # Clean up build artifacts
# make clean

# Cross-platform build configuration
VERSION ?= 0.1.0
BINARY_NAME = watchx

# Toolchain targets
toolchain:
	rustup target add x86_64-pc-windows-gnu
	rustup target add x86_64-unknown-linux-gnu

toolchain_macos:
	rustup target add x86_64-apple-darwin
	rustup target add aarch64-apple-darwin

# Windows build
build_windows:
	cargo build --release --target x86_64-pc-windows-gnu
	make zip_windows version=$(VERSION)

zip_windows:
	cd target/x86_64-pc-windows-gnu/release && \
	zip $(VERSION)_Windows_x86_64.zip $(BINARY_NAME).exe && \
	cd ../../../

# Linux build
build_linux:
	cargo build --release --target x86_64-unknown-linux-gnu
	make zip_linux version=$(VERSION)

zip_linux:
	cd target/x86_64-unknown-linux-gnu/release && \
	tar -zcvf $(VERSION)_Linux_x86_64.tar.gz $(BINARY_NAME) && \
	cd ../../../

# macOS builds
build_macos:
	cargo build --release --target x86_64-apple-darwin
	cargo build --release --target aarch64-apple-darwin
	make zip_macos_x86_64 version=$(VERSION)
	make zip_macos_arm64 version=$(VERSION)

zip_macos_x86_64:
	cd target/x86_64-apple-darwin/release && \
	tar -zcvf $(VERSION)_Darwin_x86_64.tar.gz $(BINARY_NAME) && \
	cd ../../../

zip_macos_arm64:
	cd target/aarch64-apple-darwin/release && \
	tar -zcvf $(VERSION)_Darwin_arm64.tar.gz $(BINARY_NAME) && \
	cd ../../../

# Combined builds
build_all: build_windows build_linux build_macos

# Release builds with specific version
release_windows:
	make build_windows VERSION=$(VERSION)

release_linux:
	make build_linux VERSION=$(VERSION)

release_macos:
	make build_macos VERSION=$(VERSION)

release_all:
	make build_all VERSION=$(VERSION)

# Development tools
install:
	cargo install --path .

clean:
	cargo clean
	rm -f target/**/*.zip
	rm -f target/**/*.tar.gz

.PHONY: toolchain toolchain_macos build_windows build_linux build_macos \
        zip_windows zip_linux zip_macos_x86_64 zip_macos_arm64 \
        build_all release_windows release_linux release_macos release_all \
        install clean
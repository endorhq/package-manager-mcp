TARGET ?= x86_64-unknown-linux-musl

.PHONY: build
build:
	cargo build --target=$(TARGET)

.PHONY: fmt
fmt:
	cargo fmt

.PHONY: nix-fmt
nix-fmt:
	find . -name "*.nix" | xargs alejandra

.PHONY: lint
lint:
	cargo clippy

.PHONY: test
test:
	cargo test

.PHONY: run-%
run-%: build
	docker rm -f package-manager-mcp 2> /dev/null || true
	docker run --rm -d -p 8090:8090 -v ./target/$(TARGET)/debug/package-manager-mcp:/app/package-manager-mcp --name package-manager-mcp $*:latest /app/package-manager-mcp

.PHONY: inspector-%
inspector-%: run-%
	HOST=0.0.0.0 DANGEROUSLY_OMIT_AUTH=true npx @modelcontextprotocol/inspector --config .dev-mcp.json --server apk

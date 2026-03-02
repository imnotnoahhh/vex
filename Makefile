.PHONY: help build test docs docs-stop clean install

help:
	@echo "vex - Makefile commands"
	@echo ""
	@echo "  make build       - Build release binary"
	@echo "  make test        - Run all tests"
	@echo "  make docs        - Generate and serve documentation"
	@echo "  make docs-stop   - Stop documentation server"
	@echo "  make clean       - Clean build artifacts"
	@echo "  make install     - Install to ~/.local/bin"
	@echo "  make clippy      - Run clippy linter"
	@echo "  make fmt         - Format code"
	@echo "  make bench       - Run benchmarks"

build:
	cargo build --release

test:
	cargo test --all

docs:
	@echo "Generating documentation..."
	RUSTDOCFLAGS="--html-in-header docs/header.html" cargo doc --no-deps
	cp docs/custom.css target/doc/
	@echo "Serving docs at http://localhost:8888/vex/index.html"
	@lsof -ti:8888 | xargs kill -9 2>/dev/null || true
	@cd target/doc && python3 -m http.server 8888 &
	@sleep 1
	open http://localhost:8888/vex/index.html

docs-stop:
	@lsof -ti:8888 | xargs kill 2>/dev/null && echo "Documentation server stopped" || echo "No server running"

clean:
	cargo clean

install: build
	cp target/release/vex ~/.local/bin/vex
	chmod +x ~/.local/bin/vex
	@echo "Installed to ~/.local/bin/vex"

clippy:
	cargo clippy -- -D warnings

fmt:
	cargo fmt

bench:
	cargo bench

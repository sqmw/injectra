.PHONY: help check build test test-web test-electron verify-live setup-tests run fmt doc-check

help:
	@printf '%s\n' 'make setup-tests: install locked test tools outside the repository' 'make check: format, lint, Rust/Web tests and docs' 'make build: build CLI' 'make test-electron: isolated macOS Electron smoke test' 'make run ARGS="--help": run CLI'

check: doc-check
	git diff --check
	git diff --cached --check
	sh scripts/cargo.sh fmt --all -- --check
	sh scripts/cargo.sh clippy --workspace --all-targets --locked -- -D warnings
	$(MAKE) test

build:
	sh scripts/cargo.sh build --workspace --locked

test:
	sh scripts/cargo.sh test --workspace --locked
	$(MAKE) test-web

test-web:
	node --test tests/runtime.test.mjs

setup-tests:
	node scripts/setup-tests.mjs

test-electron: build
	node scripts/electron-smoke.mjs

verify-live: build
	node scripts/verify-live.mjs $(ARGS)

run:
	sh scripts/cargo.sh run --locked -p injectra-cli -- $(ARGS)

fmt:
	sh scripts/cargo.sh fmt --all

doc-check:
	node scripts/check-docs.mjs

.PHONY: help check build test test-web test-electron verify-live setup-tests run fmt doc-check architecture-check

help:
	@printf '%s\n' 'make setup-tests: install locked test tools outside the repository' 'make check: format, lint, Rust/Web tests and docs' 'make build: build CLI' 'make test-electron: isolated macOS Electron smoke test' 'make run ARGS="--help": run CLI'

check: doc-check architecture-check
	git diff --check
	git diff --cached --check
	sh scripts/cargo.sh fmt --all -- --check
	sh scripts/cargo.sh clippy --workspace --all-targets --locked -- -D warnings
	$(MAKE) test

build:
	sh scripts/cargo.sh build --workspace --locked

test: test-webkit-host
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

architecture-check:
	node scripts/check-architecture.mjs

.PHONY: setup-frida
setup-frida:
	sh scripts/setup-frida.sh

.PHONY: test-gadget setup-gadget-tests
setup-gadget-tests:
	node scripts/setup-gadget-tests.mjs

test-gadget: build
	"$${INJECTRA_FRIDA_PYTHON:-$${INJECTRA_CACHE_DIR:-$${XDG_CACHE_HOME:-$$HOME/.cache}/injectra}/frida/venv/bin/python3}" -B scripts/gadget-smoke.py

.PHONY: setup-gadget test-frida
setup-gadget:
	python3 scripts/get-gadget.py

test-frida:
	"$${INJECTRA_FRIDA_PYTHON:-$${INJECTRA_CACHE_DIR:-$${XDG_CACHE_HOME:-$$HOME/.cache}/injectra}/frida/venv/bin/python3}" -B -m unittest discover -s tests/frida

.PHONY: setup-host-tests test-hosts
setup-host-tests:
	python3 scripts/setup-host-tests.py

test-hosts: build
	python3 -B scripts/host-smoke.py

.PHONY: test-webkit
test-webkit: build
	"$${INJECTRA_FRIDA_PYTHON:-$${INJECTRA_CACHE_DIR:-$${XDG_CACHE_HOME:-$$HOME/.cache}/injectra}/frida/venv/bin/python3}" -B scripts/webkit-smoke.py

.PHONY: test-webkit-host
test-webkit-host:
	node --test tests/webkit/host-contract.test.mjs

.PHONY: setup-gecko-tests test-gecko
setup-gecko-tests:
	/usr/bin/python3 -B scripts/setup-gecko-tests.py

test-gecko: build
	node scripts/gecko-smoke.mjs

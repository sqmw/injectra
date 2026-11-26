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


test: test-webkit-host test-gecko-html-unit
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

.PHONY: setup-thunderbird-tests test-thunderbird
setup-thunderbird-tests:
	/usr/bin/python3 -B scripts/setup-gecko-tests.py --host thunderbird

test-thunderbird:
	node scripts/thunderbird-smoke.mjs

.PHONY: test-thunderbird-html test-gecko-html-unit
test-thunderbird-html:
	node scripts/thunderbird-smoke.mjs --parsed-html

test-gecko-html-unit:
	node --test tests/gecko/parsed-html.test.mjs

.PHONY: setup-resource-tests test-resource-patch prepare-adrive-resource test-adrive-resource
setup-resource-tests:
	node scripts/setup-resource-tests.mjs

test-resource-patch:
	node --test tests/resource-patch/*.test.mjs

prepare-adrive-resource:
	node scripts/adrive-resource-smoke.mjs --prepare-local-signing $(ARGS)

test-adrive-resource:
	node scripts/adrive-resource-smoke.mjs --run-mock-keychain $(ARGS)

.PHONY: prepare-resource-app test-resource-app
prepare-resource-app:
	node scripts/resource-patch-smoke.mjs --prepare $(ARGS)

test-resource-app:
	node scripts/resource-patch-smoke.mjs --run $(ARGS)

.PHONY: setup-preparation test-preparation
setup-preparation:
	node scripts/setup-preparation-tools.mjs

test-preparation: build
	node --test crates/preparation/tests/entry.test.mjs
	node scripts/preparation-smoke.mjs

.PHONY: test-windows-format test-windows
test-windows-format:
	node --test tests/windows/pe-integrity.test.mjs tests/windows/append-asar.test.mjs tests/windows/copy-tree.test.mjs tests/windows/cef-init.test.mjs

# Windows host only: inspect-app, session attach-v8 and prepare/dev (directory/ASAR+PE/nested/ESM) on owned fixtures.
test-windows: build
	node scripts/windows-formal-smoke.mjs

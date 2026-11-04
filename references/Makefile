.PHONY: help setup program gadget check stats
include config.mk
help:
	@printf '%s\n' 'Reference archive only; excluded from product build/release.' 'setup: isolated Python/npm dependencies' 'program: build shared Web program generator' 'gadget: explicitly download verified Gadget (no app launch)' 'check: static checks, agent compilation and offline payload smoke' 'stats: count shared/app/validation/maintenance sources once'
setup:
	sh scripts/setup.sh
program:
	CARGO_TARGET_DIR="$(INJECTRA_REFERENCE_CACHE)/cargo-target" sh ../scripts/cargo.sh build --manifest-path "$(CURDIR)/shared/web-program/Cargo.toml" --locked --jobs 2
gadget:
	"$(PYTHON)" scripts/gadget.py
check: program
	"$(PYTHON)" scripts/check.py
	sh ../scripts/cargo.sh fmt --manifest-path "$(CURDIR)/shared/web-program/Cargo.toml" -- --check
	git diff --check
stats:
	node scripts/stats.mjs

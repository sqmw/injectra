.PHONY: help check

help:
	@printf '%s\n' 'make check: Git diff whitespace validation only (no product tests yet)'

check:
	git diff --check
	git diff --cached --check

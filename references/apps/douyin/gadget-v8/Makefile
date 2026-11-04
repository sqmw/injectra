include ../../../config.mk
.PHONY: launch node pages apply capture verify recover
launch:
	@test -n "$(APP_PATH)" || { echo 'APP_PATH required' >&2; exit 1; }
	"$(PYTHON)" launch.py --app "$(APP_PATH)" --proxy "$(PROXY_URL)" --code-signing required --launcher launchservices
node:
	"$(PYTHON)" node_probe.py
pages:
	"$(PYTHON)" pages.py
apply:
	"$(PYTHON)" apply_page.py
capture:
	"$(PYTHON)" capture_page.py
verify:
	"$(PYTHON)" verify_page.py
recover:
	"$(PYTHON)" recover_bootstrap.py

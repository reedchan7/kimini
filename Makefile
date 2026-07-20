# Kimini — developer convenience targets
# Usage: make [target]   (default: help)

.DEFAULT_GOAL := help

# ---------------------------------------------------------------------------
# Config
# ---------------------------------------------------------------------------
BIN       := kimini
WEB_BIN   := kimini-web
CARGO     ?= cargo
URL       ?=
BROWSER_URL ?=
DEBUG_BIN := target/debug/$(BIN)
REL_BIN   := target/release/$(BIN)
WEB_REL_BIN := target/release/$(WEB_BIN)

# ANSI (disabled when stdout is not a TTY, or when NO_COLOR is set)
ifeq ($(NO_COLOR),)
  ifeq ($(shell test -t 1 && echo yes),yes)
    C_RESET  := \033[0m
    C_BOLD   := \033[1m
    C_DIM    := \033[2m
    C_CYAN   := \033[36m
    C_GREEN  := \033[32m
    C_YELLOW := \033[33m
    C_BLUE   := \033[34m
    C_MAGENTA:= \033[35m
  endif
endif

# macOS app packaging
APP_NAME  := Kimini
WEB_APP_NAME := Kimini Web
DIST      := dist
APP_BUNDLE := $(DIST)/$(APP_NAME).app
WEB_APP_BUNDLE := $(DIST)/$(WEB_APP_NAME).app
INSTALL_DIR ?= $(HOME)/Applications
PACKAGE_SH := scripts/package-macos.sh
PUBLISH_SH := scripts/publish-release.sh
# Optional: TARGET=aarch64-apple-darwin ARCH=aarch64
TARGET    ?=
ARCH      ?=
# Extra flags for publish-release (e.g. PUBLISH_FLAGS=--dry-run)
PUBLISH_FLAGS ?=

.PHONY: help build build-web build-all release release-web release-all \
        run run-web run-release \
        check test coverage-core fmt fmt-check clippy lint clean clean-dist clean-all size \
        install install-all uninstall doctor \
        app app-native app-web apps dmg dmg-web zip zip-web package-all \
        package-linux package-linux-native package-windows publish-release \
        publish-release-all sparkle install-app install-web-app \
        uninstall-app uninstall-web-app uninstall-all \
        open-app open-web-app ship-plan

# ---------------------------------------------------------------------------
# Help (default)
# ---------------------------------------------------------------------------
##@ General

help: ## Show this help
	@printf '$(C_BOLD)$(C_CYAN)\n'
	@printf '  ██╗  ██╗██╗███╗   ███╗██╗███╗   ██╗██╗\n'
	@printf '  ██║ ██╔╝██║████╗ ████║██║████╗  ██║██║\n'
	@printf '  █████╔╝ ██║██╔████╔██║██║██╔██╗ ██║██║\n'
	@printf '  ██╔═██╗ ██║██║╚██╔╝██║██║██║╚██╗██║██║\n'
	@printf '  ██║  ██╗██║██║ ╚═╝ ██║██║██║ ╚████║██║\n'
	@printf '  ╚═╝  ╚═╝╚═╝╚═╝     ╚═╝╚═╝╚═╝  ╚═══╝╚═╝\n'
	@printf '$(C_RESET)'
	@printf '  $(C_DIM)Native Kimi Code · Web compatibility kept intact$(C_RESET)\n\n'
	@printf '  $(C_BOLD)Usage:$(C_RESET)  make $(C_GREEN)<target>$(C_RESET)  [$(C_YELLOW)VAR=value$(C_RESET) …]\n\n'
	@awk 'BEGIN { \
	    FS = ":.*##"; \
	    printf "  $(C_BOLD)Targets:$(C_RESET)\n"; \
	  } \
	  /^##@/ { \
	    printf "\n  $(C_BOLD)$(C_BLUE)%s$(C_RESET)\n", substr($$0, 5); \
	    next; \
	  } \
	  /^[a-zA-Z0-9_.-]+:.*?##/ { \
	    printf "    $(C_GREEN)%-22s$(C_RESET) %s\n", $$1, $$2; \
	  }' $(MAKEFILE_LIST)
	@printf '\n  $(C_BOLD)Variables:$(C_RESET)\n'
	@printf '    $(C_YELLOW)%-22s$(C_RESET) %s\n' 'URL' 'Kimini Web daemon URL (optional)'
	@printf '    $(C_YELLOW)%-22s$(C_RESET) %s\n' 'BROWSER_URL' 'Native human-browser start URL (optional)'
	@printf '    $(C_YELLOW)%-22s$(C_RESET) %s\n' 'CARGO' 'Cargo binary (default: cargo)'
	@printf '    $(C_YELLOW)%-22s$(C_RESET) %s\n' 'INSTALL_DIR' 'App install path (default: ~/Applications)'
	@printf '    $(C_YELLOW)%-22s$(C_RESET) %s\n' 'TARGET' 'cargo target triple (optional)'
	@printf '    $(C_YELLOW)%-22s$(C_RESET) %s\n' 'ARCH' 'Artifact arch label aarch64|x86_64 (optional)'
	@printf '    $(C_YELLOW)%-22s$(C_RESET) %s\n' 'PUBLISH_FLAGS' 'Extra flags for publish-release (e.g. --dry-run)'
	@printf '    $(C_YELLOW)%-22s$(C_RESET) %s\n' 'NO_COLOR' 'Set to disable ANSI colors in help'
	@printf '\n  $(C_DIM)Examples:$(C_RESET)\n'
	@printf '    make run BROWSER_URL='"'"'https://example.com'"'"'\n'
	@printf '    make run-web URL='"'"'http://127.0.0.1:58627/#token=…'"'"'\n'
	@printf '    make build-all\n'
	@printf '    make install-all\n'
	@printf '    make app && make install-app\n'
	@printf '    make dmg && make zip\n'
	@printf '    make package-all\n'
	@printf '    make package-linux\n'
	@printf '    make publish-release\n'
	@printf '    make publish-release-all\n'
	@printf '    make publish-release PUBLISH_FLAGS=--dry-run\n'
	@printf '    make lint\n\n'

# ---------------------------------------------------------------------------
# All-in-one (batch convenience — see individual groups for finer targets)
# ---------------------------------------------------------------------------
##@ All-in-one

build-all: build build-web ## Debug-build native + web
release-all: release release-web ## Release-build native + web
apps: app-native app-web ## Build both macOS apps (dist/*.app)
install-all: install install-app install-web-app ## Install CLI + both .apps
uninstall-all: uninstall uninstall-app uninstall-web-app ## Remove CLI + both .apps
package-all: ## Dual-arch DMG + zip for both apps into dist/
	@test "$$(uname -s)" = "Darwin" || { echo "error: package-all requires macOS"; exit 1; }
	bash ./$(PACKAGE_SH) --app native --target aarch64-apple-darwin --arch aarch64 --dmg --zip
	bash ./$(PACKAGE_SH) --app web --target aarch64-apple-darwin --arch aarch64 --dmg --zip
	bash ./$(PACKAGE_SH) --app native --target x86_64-apple-darwin --arch x86_64 --dmg --zip
	bash ./$(PACKAGE_SH) --app web --target x86_64-apple-darwin --arch x86_64 --dmg --zip
	@ls -lh '$(DIST)'/*.dmg '$(DIST)'/*.zip 2>/dev/null || true

publish-release-all: ## Publish macOS + Linux + staged Windows asset matrix
	@test "$$(uname -s)" = "Darwin" || { echo "error: publish-release-all coordinator requires macOS"; exit 1; }
	bash ./$(PUBLISH_SH) --include-portable $(PUBLISH_FLAGS)

clean-all: clean clean-dist ## cargo clean + remove dist/

ship-plan: ## Read-only ship plan (BUMP=default|patch|minor|major|X.Y.Z)
	bash ./scripts/ship.sh plan --bump '$(or $(BUMP),default)'

# ---------------------------------------------------------------------------
# Build
# ---------------------------------------------------------------------------
##@ Build

build: ## Debug build
	$(CARGO) build --bin $(BIN) --no-default-features --features native

build-web: ## Debug build for Kimini Web
	$(CARGO) build --bin $(WEB_BIN) --no-default-features --features legacy-web

release: ## Optimized release build
	$(CARGO) build --release --bin $(BIN) --no-default-features --features native

release-web: ## Release build for Kimini Web
	$(CARGO) build --release --bin $(WEB_BIN) --no-default-features --features legacy-web

# ---------------------------------------------------------------------------
# macOS Application
# ---------------------------------------------------------------------------
##@ macOS App

sparkle: ## Download the pinned Sparkle framework and release tools
	@test "$$(uname -s)" = "Darwin" || { echo "error: Sparkle requires macOS"; exit 1; }
	bash ./scripts/fetch-sparkle.sh

# Extra flags forwarded to package-macos.sh when TARGET/ARCH set
PACKAGE_FLAGS :=
ifneq ($(TARGET),)
  PACKAGE_FLAGS += --target $(TARGET)
endif
ifneq ($(ARCH),)
  PACKAGE_FLAGS += --arch $(ARCH)
endif

app: app-native ## Build release + package dist/Kimini.app

app-native: ## Build native dist/Kimini.app
	@test "$$(uname -s)" = "Darwin" || { echo "error: app packaging requires macOS"; exit 1; }
	bash ./$(PACKAGE_SH) --app native $(PACKAGE_FLAGS)

app-web: ## Build legacy dist/Kimini Web.app
	@test "$$(uname -s)" = "Darwin" || { echo "error: app packaging requires macOS"; exit 1; }
	bash ./$(PACKAGE_SH) --app web $(PACKAGE_FLAGS)

dmg: ## Build .app + DMG (Kimini-<ver>-macos-<arch>.dmg)
	@test "$$(uname -s)" = "Darwin" || { echo "error: dmg packaging requires macOS"; exit 1; }
	bash ./$(PACKAGE_SH) --app native $(PACKAGE_FLAGS) --dmg

dmg-web: ## Build Kimini Web DMG
	@test "$$(uname -s)" = "Darwin" || { echo "error: dmg packaging requires macOS"; exit 1; }
	bash ./$(PACKAGE_SH) --app web $(PACKAGE_FLAGS) --dmg

zip: ## Build .app + zip archive for distribution
	@test "$$(uname -s)" = "Darwin" || { echo "error: zip packaging requires macOS"; exit 1; }
	bash ./$(PACKAGE_SH) --app native $(PACKAGE_FLAGS) --zip

zip-web: ## Build Kimini Web zip archive
	@test "$$(uname -s)" = "Darwin" || { echo "error: zip packaging requires macOS"; exit 1; }
	bash ./$(PACKAGE_SH) --app web $(PACKAGE_FLAGS) --zip

publish-release: ## Local dual-arch package + GitHub Release (version from Cargo.toml)
	@test "$$(uname -s)" = "Darwin" || { echo "error: publish-release requires macOS"; exit 1; }
	bash ./$(PUBLISH_SH) $(PUBLISH_FLAGS)

# ---------------------------------------------------------------------------
# Linux / Windows portable packages
# ---------------------------------------------------------------------------
##@ Linux / Windows

package-linux: ## Build Linux x86_64 + ARM64 archives through Docker
	bash ./scripts/package-linux-docker.sh --arch all

package-linux-native: ## Build Linux archives on the current Linux host
	bash ./scripts/package-linux.sh --app all

package-windows: ## Build Windows x86_64 + ARM64 archives in Developer PowerShell
	powershell.exe -NoProfile -ExecutionPolicy Bypass -File ./scripts/package-windows.ps1 -App all -Arch all

##@ macOS Install

install-app: ## Package and install .app to INSTALL_DIR (default: ~/Applications)
	@test "$$(uname -s)" = "Darwin" || { echo "error: install-app requires macOS"; exit 1; }
	bash ./$(PACKAGE_SH) --app native $(PACKAGE_FLAGS) --install "$(INSTALL_DIR)"

install-web-app: ## Package and install Kimini Web.app
	@test "$$(uname -s)" = "Darwin" || { echo "error: install-web-app requires macOS"; exit 1; }
	bash ./$(PACKAGE_SH) --app web $(PACKAGE_FLAGS) --install "$(INSTALL_DIR)"

uninstall-app: ## Remove Kimini.app from INSTALL_DIR (default: ~/Applications)
	@if [ -d "$(INSTALL_DIR)/$(APP_NAME).app" ]; then \
	  rm -rf "$(INSTALL_DIR)/$(APP_NAME).app"; \
	  printf 'removed %s\n' "$(INSTALL_DIR)/$(APP_NAME).app"; \
	else \
	  printf 'not installed: %s\n' "$(INSTALL_DIR)/$(APP_NAME).app"; \
	fi

uninstall-web-app: ## Remove Kimini Web.app from INSTALL_DIR
	@if [ -d "$(INSTALL_DIR)/$(WEB_APP_NAME).app" ]; then \
	  rm -rf "$(INSTALL_DIR)/$(WEB_APP_NAME).app"; \
	  printf 'removed %s\n' "$(INSTALL_DIR)/$(WEB_APP_NAME).app"; \
	else \
	  printf 'not installed: %s\n' "$(INSTALL_DIR)/$(WEB_APP_NAME).app"; \
	fi

open-app: ## Open native packaged app; BROWSER_URL='…' opens the browser pane
	@test "$$(uname -s)" = "Darwin" || { echo "error: open-app requires macOS"; exit 1; }
	@if [ ! -d '$(APP_BUNDLE)' ]; then bash ./$(PACKAGE_SH) --app native $(PACKAGE_FLAGS); fi
	@if [ -n '$(BROWSER_URL)' ]; then \
	  open -na '$(APP_BUNDLE)' --env KIMINI_BROWSER_URL='$(BROWSER_URL)'; \
	else \
	  open '$(APP_BUNDLE)'; \
	fi

open-web-app: ## Open Kimini Web.app (builds if missing)
	@test "$$(uname -s)" = "Darwin" || { echo "error: open-app requires macOS"; exit 1; }
	@if [ ! -d '$(WEB_APP_BUNDLE)' ]; then bash ./$(PACKAGE_SH) --app web $(PACKAGE_FLAGS); fi
	@if [ -n '$(URL)' ]; then \
	  open -na '$(WEB_APP_BUNDLE)' --args '$(URL)'; \
	else \
	  open '$(WEB_APP_BUNDLE)'; \
	fi

# ---------------------------------------------------------------------------
# Run
# ---------------------------------------------------------------------------
##@ Run

run: ## Run the native app
	@if [ -n '$(BROWSER_URL)' ]; then \
	  KIMINI_BROWSER_URL='$(BROWSER_URL)' $(CARGO) run --bin $(BIN) --no-default-features --features native; \
	else \
	  $(CARGO) run --bin $(BIN) --no-default-features --features native; \
	fi

run-web: ## Run Kimini Web; pass URL='…' on first auth
	$(CARGO) run --bin $(WEB_BIN) --no-default-features --features legacy-web -- $(URL)

run-release: release ## Run the native release binary
	@if [ -n '$(BROWSER_URL)' ]; then \
	  KIMINI_BROWSER_URL='$(BROWSER_URL)' ./$(REL_BIN); \
	else \
	  ./$(REL_BIN); \
	fi

# ---------------------------------------------------------------------------
# Quality
# ---------------------------------------------------------------------------
##@ Quality

check: ## Type-check both applications
	$(CARGO) check --all-targets --all-features

test: ## Run tests
	$(CARGO) test --all-targets --all-features

coverage-core: ## Enforce 90% line coverage for protocol and state logic
	$(CARGO) llvm-cov --all-features --all-targets --summary-only \
	  --ignore-filename-regex 'src/(api|bin|daemon|i18n|legacy_web|native)/|src/(i18n|updater)\.rs' \
	  --fail-under-lines 90

fmt: ## Format sources with rustfmt
	$(CARGO) fmt

fmt-check: ## Check formatting (no write)
	$(CARGO) fmt -- --check

clippy: ## Lint with clippy (deny warnings)
	$(CARGO) clippy --all-targets --all-features -- -D warnings

lint: fmt-check clippy ## Format check + clippy

# ---------------------------------------------------------------------------
# Utilities
# ---------------------------------------------------------------------------
##@ Utilities

clean: ## Remove cargo target artifacts (keeps dist/)
	$(CARGO) clean

clean-dist: ## Remove dist/ packaging outputs
	rm -rf '$(DIST)'

size: ## Print debug/release binary and .app sizes (if present)
	@printf '$(C_BOLD)Binary sizes$(C_RESET)\n'
	@if [ -f '$(DEBUG_BIN)' ]; then \
	  printf '  $(C_CYAN)debug$(C_RESET)   %s\n' "$$(du -h '$(DEBUG_BIN)' | cut -f1)  ($(DEBUG_BIN))"; \
	else \
	  printf '  $(C_DIM)debug   (not built — make build)$(C_RESET)\n'; \
	fi
	@if [ -f '$(REL_BIN)' ]; then \
	  printf '  $(C_CYAN)release$(C_RESET) %s\n' "$$(du -h '$(REL_BIN)' | cut -f1)  ($(REL_BIN))"; \
	  printf '  $(C_DIM)bytes   %s$(C_RESET)\n' "$$(wc -c < '$(REL_BIN)' | tr -d ' ')"; \
	else \
	  printf '  $(C_DIM)release (not built — make release)$(C_RESET)\n'; \
	fi
	@if [ -d '$(APP_BUNDLE)' ]; then \
	  printf '  $(C_CYAN)app$(C_RESET)     %s\n' "$$(du -sh '$(APP_BUNDLE)' | cut -f1)  ($(APP_BUNDLE))"; \
	else \
	  printf '  $(C_DIM)app     (not packaged — make app)$(C_RESET)\n'; \
	fi

install: release ## Install release binary to ~/.cargo/bin
	$(CARGO) install --path . --force

uninstall: ## Remove cargo-installed kimini binaries from ~/.cargo/bin
	@if $(CARGO) uninstall kimini >/dev/null 2>&1; then \
	  printf 'removed cargo package kimini (~/.cargo/bin)\n'; \
	else \
	  printf 'not installed: kimini (cargo)\n'; \
	fi

doctor: ## Show toolchain / pinned dep versions
	@printf '$(C_BOLD)Environment$(C_RESET)\n'
	@printf '  $(C_CYAN)rustc$(C_RESET)  '; rustc --version
	@printf '  $(C_CYAN)cargo$(C_RESET)  '; $(CARGO) --version
	@printf '  $(C_CYAN)host$(C_RESET)   '; rustc -vV | awk '/^host:/{print $$2}'
	@printf '\n$(C_BOLD)Pinned stack (Cargo.toml)$(C_RESET)\n'
	@awk '/^tao =|^wry =|^muda /{print "  " $$0}' Cargo.toml
	@printf '\n$(C_BOLD)Paths$(C_RESET)\n'
	@printf '  $(C_CYAN)debug$(C_RESET)   $(DEBUG_BIN)\n'
	@printf '  $(C_CYAN)release$(C_RESET) $(REL_BIN)\n'
	@printf '  $(C_CYAN)app$(C_RESET)     $(APP_BUNDLE)\n'
	@printf '  $(C_CYAN)install$(C_RESET) $(INSTALL_DIR)/$(APP_NAME).app\n'
	@printf '  $(C_CYAN)default$(C_RESET) URL http://127.0.0.1:58627/\n'

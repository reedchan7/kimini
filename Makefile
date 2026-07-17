# Kimini — developer convenience targets
# Usage: make [target]   (default: help)

.DEFAULT_GOAL := help

# ---------------------------------------------------------------------------
# Config
# ---------------------------------------------------------------------------
BIN       := kimini
CARGO     ?= cargo
URL       ?=
DEBUG_BIN := target/debug/$(BIN)
REL_BIN   := target/release/$(BIN)

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
DIST      := dist
APP_BUNDLE := $(DIST)/$(APP_NAME).app
INSTALL_DIR ?= $(HOME)/Applications
PACKAGE_SH := scripts/package-macos.sh
PUBLISH_SH := scripts/publish-release.sh
# Optional: TARGET=aarch64-apple-darwin ARCH=aarch64
TARGET    ?=
ARCH      ?=
# Extra flags for publish-release (e.g. PUBLISH_FLAGS=--dry-run)
PUBLISH_FLAGS ?=

.PHONY: help build release run run-release check test fmt fmt-check clippy lint \
        clean clean-dist size install doctor app dmg zip package-all \
        publish-release install-app open-app

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
	@printf '  $(C_DIM)The lightest way to browse. · Kimi Code Web shell$(C_RESET)\n\n'
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
	    printf "    $(C_GREEN)%-16s$(C_RESET) %s\n", $$1, $$2; \
	  }' $(MAKEFILE_LIST)
	@printf '\n  $(C_BOLD)Variables:$(C_RESET)\n'
	@printf '    $(C_YELLOW)%-16s$(C_RESET) %s\n' 'URL' 'Start URL for run/run-release/open-app (optional)'
	@printf '    $(C_YELLOW)%-16s$(C_RESET) %s\n' 'CARGO' 'Cargo binary (default: cargo)'
	@printf '    $(C_YELLOW)%-16s$(C_RESET) %s\n' 'INSTALL_DIR' 'App install path (default: ~/Applications)'
	@printf '    $(C_YELLOW)%-16s$(C_RESET) %s\n' 'TARGET' 'cargo target triple (optional)'
	@printf '    $(C_YELLOW)%-16s$(C_RESET) %s\n' 'ARCH' 'Artifact arch label aarch64|x86_64 (optional)'
	@printf '    $(C_YELLOW)%-16s$(C_RESET) %s\n' 'PUBLISH_FLAGS' 'Extra flags for publish-release (e.g. --dry-run)'
	@printf '    $(C_YELLOW)%-16s$(C_RESET) %s\n' 'NO_COLOR' 'Set to disable ANSI colors in help'
	@printf '\n  $(C_DIM)Examples:$(C_RESET)\n'
	@printf '    make run URL='"'"'http://127.0.0.1:58627/#token=…'"'"'\n'
	@printf '    make app && make install-app\n'
	@printf '    make dmg && make zip\n'
	@printf '    make package-all\n'
	@printf '    make publish-release\n'
	@printf '    make publish-release PUBLISH_FLAGS=--dry-run\n'
	@printf '    make lint\n\n'

# ---------------------------------------------------------------------------
# Build
# ---------------------------------------------------------------------------
##@ Build

build: ## Debug build
	$(CARGO) build

release: ## Release build (LTO + size-optimized)
	$(CARGO) build --release

# ---------------------------------------------------------------------------
# macOS Application
# ---------------------------------------------------------------------------
##@ macOS App

# Extra flags forwarded to package-macos.sh when TARGET/ARCH set
PACKAGE_FLAGS :=
ifneq ($(TARGET),)
  PACKAGE_FLAGS += --target $(TARGET)
endif
ifneq ($(ARCH),)
  PACKAGE_FLAGS += --arch $(ARCH)
endif

app: ## Build release + package dist/Kimini.app
	@test "$$(uname -s)" = "Darwin" || { echo "error: app packaging requires macOS"; exit 1; }
	./$(PACKAGE_SH) $(PACKAGE_FLAGS)

dmg: ## Build .app + DMG (Kimini-<ver>-macos-<arch>.dmg)
	@test "$$(uname -s)" = "Darwin" || { echo "error: dmg packaging requires macOS"; exit 1; }
	./$(PACKAGE_SH) $(PACKAGE_FLAGS) --dmg

zip: ## Build .app + zip archive for distribution
	@test "$$(uname -s)" = "Darwin" || { echo "error: zip packaging requires macOS"; exit 1; }
	./$(PACKAGE_SH) $(PACKAGE_FLAGS) --zip

package-all: ## Build aarch64 + x86_64 DMG and zip into dist/
	@test "$$(uname -s)" = "Darwin" || { echo "error: package-all requires macOS"; exit 1; }
	./$(PACKAGE_SH) --target aarch64-apple-darwin --arch aarch64 --dmg --zip
	./$(PACKAGE_SH) --target x86_64-apple-darwin --arch x86_64 --dmg --zip
	@ls -lh '$(DIST)'/*.dmg '$(DIST)'/*.zip 2>/dev/null || true

publish-release: ## Local dual-arch package + GitHub Release (version from Cargo.toml)
	@test "$$(uname -s)" = "Darwin" || { echo "error: publish-release requires macOS"; exit 1; }
	./$(PUBLISH_SH) $(PUBLISH_FLAGS)

install-app: ## Package and install .app to INSTALL_DIR (default: ~/Applications)
	@test "$$(uname -s)" = "Darwin" || { echo "error: install-app requires macOS"; exit 1; }
	./$(PACKAGE_SH) $(PACKAGE_FLAGS) --install "$(INSTALL_DIR)"

open-app: ## Open the packaged app (builds if missing); pass URL='…' on first auth
	@test "$$(uname -s)" = "Darwin" || { echo "error: open-app requires macOS"; exit 1; }
	@if [ ! -d '$(APP_BUNDLE)' ]; then ./$(PACKAGE_SH) $(PACKAGE_FLAGS); fi
	@if [ -n '$(URL)' ]; then \
	  open -na '$(APP_BUNDLE)' --args '$(URL)'; \
	else \
	  open '$(APP_BUNDLE)'; \
	fi

# ---------------------------------------------------------------------------
# Run
# ---------------------------------------------------------------------------
##@ Run

run: ## Run debug build; pass URL='…' on first auth
	$(CARGO) run -- $(URL)

run-release: release ## Run release binary; pass URL='…' on first auth
	./$(REL_BIN) $(URL)

# ---------------------------------------------------------------------------
# Quality
# ---------------------------------------------------------------------------
##@ Quality

check: ## Type-check without producing binaries
	$(CARGO) check

test: ## Run tests
	$(CARGO) test

fmt: ## Format sources with rustfmt
	$(CARGO) fmt

fmt-check: ## Check formatting (no write)
	$(CARGO) fmt -- --check

clippy: ## Lint with clippy (deny warnings)
	$(CARGO) clippy --all-targets -- -D warnings

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

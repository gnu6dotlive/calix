# ──────────────────────────────────────────────────────────────────────────────
# Calix GPUI — Makefile de développement
# Usage:
#   make run       — compile (dev) et lance l'app avec le CLI Zig local
#   make build     — cargo build --release
#   make check     — cargo check (rapide, sans lier)
#   make zig       — (re)compile les binaires Zig
#   make clean     — nettoie les artefacts Rust et Zig
#   make package   — construit le paquet dans un répertoire isolé (sans installer)
#   make package-install — construit puis installe le paquet via pkexec
#   make install   — copie les binaires en /usr/local (sans PKGBUILD)
# ──────────────────────────────────────────────────────────────────────────────

REPO_ROOT   := $(dir $(abspath $(lastword $(MAKEFILE_LIST))))
ZIG_SRC     := $(REPO_ROOT)Calix.Cli.Zig
ZIG_BIN        := $(ZIG_SRC)/zig-out/bin/calix
ZIG_NOTIFY_BIN := $(ZIG_SRC)/zig-out/bin/calix-notify
RUST_SRC       := $(REPO_ROOT)Calix.Ui.Gpui
TARGET_DIR     := $(shell cd $(RUST_SRC) && cargo metadata --format-version 1 --no-deps 2>/dev/null | grep -o '"target_directory":"[^"]*"' | cut -d'"' -f4)
ifeq ($(TARGET_DIR),)
  TARGET_DIR := $(RUST_SRC)/target
endif
RUST_BIN    := $(TARGET_DIR)/release/calix-gpui
PKGBUILD    := $(REPO_ROOT)PKGBUILD-calix
# Zig 0.16 is required by the build API. Prefer the git-ignored vendored
# toolchain when present; package builds receive the same path via ZIG.
ZIG ?= $(if $(wildcard $(REPO_ROOT)vendor/zig-0.16/usr/bin/zig),$(REPO_ROOT)vendor/zig-0.16/usr/bin/zig,zig)
export ZIG

# Préfère le CLI Zig local ; tombe en arrière sur le CLI système si absent
ifeq ($(wildcard $(ZIG_BIN)),)
  CALIX_BIN ?= $(shell which calix 2>/dev/null)
else
  CALIX_BIN ?= $(ZIG_BIN)
endif

export CALIX_BIN

# ─── Cibles principales ───────────────────────────────────────────────────────

.PHONY: run build check test dependencies dependencies-update zig clean package package-install install help

## Lance l'application en mode développement (cargo run)
run: zig
	@echo ">>> CALIX_BIN = $(CALIX_BIN)"
	@echo ">>> Lancement de calix-gpui..."
	cd $(RUST_SRC) && CALIX_BIN=$(CALIX_BIN) cargo run 2>&1

## Compile le binaire de release Rust
build: zig
	@echo ">>> Build release..."
	cd $(RUST_SRC) && CALIX_BIN=$(CALIX_BIN) cargo build --release
	@echo ">>> Binaire : $(RUST_BIN)"

## Vérifie le code Rust sans lier (rapide)
check:
	cd $(RUST_SRC) && cargo check

## Lance la suite de tests unitaires Rust
test:
	cd $(RUST_SRC) && cargo test

## L'état des dépendances : ce qui est en retard, et les épingles connues
dependencies:
	@./scripts/dependencies

## Met à jour le lockfile, repose les épingles, et laisse la porte décider
dependencies-update:
	@./scripts/dependencies --update

## Compile les binaires Zig
zig:
	@if [ ! -f "$(ZIG_BIN)" ] || [ ! -f "$(ZIG_NOTIFY_BIN)" ]; then \
		echo ">>> Compilation des binaires Zig (ReleaseSafe)..."; \
		cd $(ZIG_SRC) && $(ZIG) build -Doptimize=ReleaseSafe; \
	else \
		echo ">>> Binaires Zig déjà compilés : $(ZIG_BIN)"; \
	fi

## Force la recompilation des binaires Zig
zig-rebuild:
	@echo ">>> Recompilation forcée des binaires Zig..."
	cd $(ZIG_SRC) && $(ZIG) build -Doptimize=ReleaseSafe

## Construit le paquet dans /tmp sans installer (jamais sudo)
# Le travail de la construction vit dans le dépôt, dans le répertoire que le
# dépôt ignore : /tmp est en mémoire et partagé.
PKG_WORKDIR := $(REPO_ROOT).ci-cache/pkg

package:
	@echo ">>> Test PKGBUILD dans $(PKG_WORKDIR)..."
	rm -rf $(PKG_WORKDIR)
	mkdir -p $(PKG_WORKDIR)
	cp $(PKGBUILD) $(PKG_WORKDIR)/PKGBUILD
	cd $(PKG_WORKDIR) && makepkg --noconfirm

## Installe le paquet construit via polkit
package-install: package
	@echo ">>> Installation via pkexec..."
	pkexec pacman -U --noconfirm $(PKG_WORKDIR)/*.pkg.tar.zst

## Installe directement sans PKGBUILD (pour dev rapide)
install: build
	@echo ">>> Installation dans /usr/local..."
	install -Dm755 $(RUST_BIN) /usr/local/lib/calix/calix-gpui
	install -Dm755 $(ZIG_BIN)  /usr/local/lib/calix/calix
	@printf '#!/bin/sh\nCALIX_BIN=/usr/local/lib/calix/calix exec /usr/local/lib/calix/calix-gpui "$$@"\n' \
		| install -Dm755 /dev/stdin /usr/local/bin/calix-gpui
	install -Dm644 $(RUST_SRC)/assets/live.gnu6.calix.desktop \
		/usr/local/share/applications/live.gnu6.calix.desktop
	install -Dm644 $(RUST_SRC)/assets/live.gnu6.calix.svg \
		/usr/local/share/icons/hicolor/scalable/apps/live.gnu6.calix.svg
	@echo ">>> Installé. Lance : calix-gpui"

## Nettoie tous les artefacts compilés
clean:
	cd $(RUST_SRC) && cargo clean
	rm -rf $(ZIG_SRC)/zig-out $(ZIG_SRC)/.zig-cache

## Affiche cette aide
help:
	@grep -E '^##' $(MAKEFILE_LIST) | sed 's/## /  /'

.DEFAULT_GOAL := help

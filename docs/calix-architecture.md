# 🏺 Calix — Master Plan
> Gestionnaire de paquets universel pour gnosix desktop  
> Base: `~/Projects/shelly-gpui` (renommé in-place, historique préservé)  
> Date: 2026-10-19

---

## 🗺 Vision

**Calix** est le hub central de gestion de paquets du bureau gnosix. Il remplace Shelly officiel (`/usr/bin/shelly`) et unifie tous les canaux d'installation en un seul outil cohérent : GPU-accéléré, self-healing, daemon-driven, et prêt pour l'intégration LLM-agent.

> [!IMPORTANT]
> Calix n'est pas un fork de Shelly — c'est une continuation de Shelly-GPUI, évoluant vers un projet indépendant. Le code GPUI, le backend Zig, l'architecture AppImage/AUR/Flatpak existants sont la fondation.

---

## 🔬 CachyOS Research Findings (2026-10-19)

### Ce qui va dans notre sens ✅
- **Shelly v3.1.x est lui-même écrit en Zig** — notre `calix.cli.zig` est convergent avec upstream, pas excentrique.
- **Btrfs est bien en place** sur `/` et `/home` (CachyOS) et **Snapper 0.13.2 est installé** avec la config `root`.
  > [!CAUTION]
  > **Correction 2026-10-19 :** le hook `snap-pac` **n'est PAS installé** (`/etc/pacman.d/hooks/` ne contient que
  > `90-mkinitcpio-install`, `95-nvidia-reboot-marker`, `99-cachyos-hello-patch`). **Aucune transaction pacman ne
  > crée de snapshot automatique** — contrairement à ce qu'affirmait la version précédente de ce plan. Les
  > snapshots pré-update doivent être **créés manuellement** (`pkexec snapper -c root create --description "…"`),
  > ou `snap-pac` doit être installé pour rétablir la garantie.
- **Hyprland est flagship chez CachyOS** (`cachyos-hypr-noctalia`) — intégration XDG `.desktop` parfaitement supportée.

### Risques / Contraintes identifiés ⚠️
- **`cachy-update` tray conflict** = source probable des erreurs Shelly actuelles. Le daemon tray `cachy-update` et Shelly se battent pour le lock pacman DB (`/var/lib/pacman/db.lck`). **Action immédiate** : désactiver `cachy-update` quand Calix prend le relais.
- **Homebrew** : aucune stance CachyOS — backend `brew` documenté comme `UNSUPPORTED_BY_DISTRO` (best-effort).
- **AppImage : `fuse2` non bundlé par défaut** — Calix doit vérifier/installer `fuse2` au premier lancement AppImage.
- **AM/AppMan** (`github.com/ivan-hc/AM`) = "AUR pour AppImages", recommandé par CachyOS. Évaluer comme source d'inspiration pour `Calix.Release`.
- **Snap** : explicitement découragé par CachyOS — pas de backend Snap prévu dans Calix.

### Alignment Table
| Aspect | CachyOS Direction | Calix Alignment |
|--------|-----------------|----------------|
| Package priority | CachyOS repos > Arch > AUR > Flatpak > AppImage | ✅ Même ordre |
| AUR helper | Shelly CLI (paru/yay installables) | ✅ libalpm + AUR backend |
| Btrfs snapshots | snap-pac + Snapper (déjà actifs) | ✅ S'intègre via Snapper API |
| AppImage mgmt | GearLever / AM/AppMan (community) | 🔄 Calix.Release remplace + étend |
| Snap | Explicitement découragé | ✅ Aucun backend snap prévu |
| Homebrew | Pas de stance officielle | ⚠️ Best-effort, non-supporté distro |
| Hyprland | Flagship (noctalia) | ✅ XDG .desktop natif |

---

## 🏗 Architecture Cible

```
┌──────────────────────────────────────────────────────────────┐
│                    calix.ui.gpui (Rust)                      │
│          GPUI/Blade · Wayland-natif · 120+ FPS               │
│  ┌──────────┐ ┌─────────┐ ┌──────────┐ ┌─────────────────┐  │
│  │ Search   │ │Inspector│ │ Log TUI  │ │  Notif. Wayland │  │
│  └──────────┘ └─────────┘ └──────────┘ └─────────────────┘  │
└──────────────────────────┬───────────────────────────────────┘
                           │ IPC / Unix Socket
┌──────────────────────────▼───────────────────────────────────┐
│               calix-sync.service (daemon systemd user)        │
│           Source of Truth · État temps réel · SQLite agent   │
└──────────────────────────┬───────────────────────────────────┘
                           │
┌──────────────────────────▼───────────────────────────────────┐
│                  calix.cli.zig (Zig backend)                  │
│  ┌─────────┐ ┌─────┐ ┌──────────┐ ┌─────┐ ┌──────────────┐  │
│  │  ALPM   │ │ AUR │ │ Flatpak  │ │ AppI│ │ Calix.Release│  │
│  │ pacman  │ │ yay │ │(optionnel│ │mage │ │ GitHub/GitLab│  │
│  └─────────┘ └─────┘ └──────────┘ └─────┘ └──────────────┘  │
│  ┌──────────┐ ┌─────┐ ┌──────────┐ ┌─────────────────────┐  │
│  │   npm   │ │ pip │ │  cargo   │ │    Homebrew/brew     │  │
│  │ (global)│ │/pipx│ │(install) │ │    (linuxbrew)       │  │
│  └──────────┘ └─────┘ └──────────┘ └─────────────────────┘  │
└──────────────────────────┬───────────────────────────────────┘
                           │
┌──────────────────────────▼───────────────────────────────────┐
│                     SurrealDB (embedded)                      │
│    Package registry · Update history · Lockfile · LLM-ready  │
└──────────────────────────────────────────────────────────────┘
```

---

## 📦 Backends

| Backend | Source actuelle | Priorité | Statut dans GPUI |
|---------|----------------|----------|-----------------|
| `alpm` | pacman (libalpm) | P0 | ✅ Existant |
| `aur` | yay / git clone | P0 | ✅ Existant |
| `appimage` | `~/Applications/` | P0 | ✅ Existant (partiel) |
| `flatpak` | Flathub | P1 | ✅ Existant |
| `release` | GitHub/GitLab/Gitea | P0 | 🔴 À créer |
| `npm` | `/usr/lib` global | P1 | 🔴 À créer |
| `pip` / `pipx` | Python env | P1 | 🔴 À créer |
| `cargo` | `~/.cargo/bin/` | P1 | 🔴 À créer |
| `brew` | Homebrew/linuxbrew | P2 | 🔴 À créer |
| `manual` | curl/wget binaires | P2 | 🔴 À créer |

---

## 🔭 Calix.Release — Le Probing de Repos Publics

Nouveau backend `Calix.Release` dans `calix.cli.zig` :

### Fonctionnement
1. **Input** : URL d'un repo public (GitHub, GitLab, Gitea, lien direct)
2. **Probe** : interroge l'API publique pour lister les releases/tags/canaux (stable, beta, canary, nightly)
3. **Asset Resolution** (ordre de préférence) :
   ```
   AppImage > Flatpak > tar.gz > .deb/.rpm (extrait) > source (build)
   ```
4. **Install** : télécharge, vérifie checksum, installe dans `~/Applications/`, crée `.desktop` XDG
5. **Lockfile** : enregistre dans SurrealDB (`source_url`, `version`, `canal`, `asset_type`, `checksum`)
6. **Update** : compare la version lockée vs. latest release à chaque `calix-sync` poll

### Lockfile SurrealDB Schema (simplifié)
```surql
DEFINE TABLE package SCHEMALESS;
-- Exemple d'entrée release
CREATE package:artcraft_launcher SET
  name = "artcraft-launcher",
  backend = "release",
  source = "https://github.com/gnosix/artcraft-launcher",
  canal = "stable",
  version = "0.4.1",
  asset_type = "tar.gz",
  checksum = "sha256:...",
  install_path = "/usr/local/bin/artcraft-launcher",
  desktop_file = "~/.local/share/applications/artcraft-launcher.desktop",
  installed_at = time::now(),
  updated_at = time::now();
```

---

## 🗄 Base de Données : SurrealDB Embedded

**Pourquoi SurrealDB** : graph + document + SQL hybride, embedded mode (pas de serveur), extensible pour les agents LLM nano-task futurs.

```
~/.local/share/calix/
├── calix.db           ← SurrealDB embedded data
├── calix.db.wal       ← Write-Ahead Log
└── snapshots/         ← Snapshots pré-update (JSON exports)
```

**Tables principales** :
- `package` — tous les paquets installés, tous backends confondus
- `update_event` — historique de chaque mise à jour (avec diff de version)
- `rollback_checkpoint` — checkpoints pré-update avec path du snapshot Btrfs
- `backend_config` — configuration par canal (canaux npm, pip env, brew prefix...)

---

## 🌊 calix-sync.service — Daemon Source of Truth

```ini
# ~/.config/systemd/user/calix-sync.service
[Unit]
Description=Calix Package State Daemon
After=network.target

[Service]
Type=notify
ExecStart=%h/.local/bin/calix sync --daemon
Restart=on-failure
RestartSec=10

[Install]
WantedBy=default.target
```

**Cycle du daemon :**
1. **Au démarrage** : scan complet de tous les backends → état initial SurrealDB
2. **Poll continu** (configurable, défaut 4h) : compare versions installées vs. disponibles
3. **Détection de changements externes** : inotify sur `~/Applications/`, `~/.cargo/bin/`, etc.
4. **Fenêtre de mise à jour** : configurable (ex: 02:00–04:00 local) ou sur login
5. **Notification Wayland** avant d'appliquer : `notify-send` / portal DBus
6. **Expose IPC socket** : `~/.local/run/calix.sock` → consommé par `calix.ui.gpui`

---

## 🛡 Self-Healing & Rollback

### Stratégie par couche

| Canal | Snapshot pre-update | Rollback |
|-------|---------------------|---------|
| pacman / AUR | `btrfs subvolume snapshot` (si Btrfs) | `btrfs snapshot restore` |
| AppImage | Copie de l'ancien `.AppImage` dans `~/.local/share/calix/snapshots/` | Restauration directe |
| Calix.Release | Lockfile entry + binaire backup | Réinstall depuis backup |
| npm / pip / cargo | Lockfile + version pinée | `npm install@version`, `pip install==ver` |
| brew | `brew pin` + `brew switch` | `brew switch formula version` |

### Flow en cas d'échec
```
1. Update commence → snapshot créé
2. Update échoue → retry automatique (1x)
3. Si retry échoue → rollback auto depuis snapshot
4. Notification Wayland : "Mise à jour de X échouée, rollback appliqué"
5. Log détaillé dans SurrealDB + fichier log ~/.local/state/calix/errors.log
```

---

## 📱 Intégration AppImages XDG

Pour chaque AppImage dans `~/Applications/` :

```
1. Calix monte l'AppImage en lecture et extrait :
   - AppStream metadata (si présent) → version, description, icône
   - Fallback : nom du fichier → version via regex (ex: tty7-26.8.4-nightly.*.AppImage)
2. Crée ~/.local/share/applications/{nom}.desktop
3. Extrait l'icône → ~/.local/share/icons/hicolor/
4. Met à jour la DB : version installée, source (si connue), canal
```

> [!NOTE]
> Artcraft Launcher gère les crafts apps mais **ne peut pas se mettre à jour lui-même**. Calix aura un backend `release` pointant vers le repo Artcraft Launcher, géré indépendamment.

---

## 🔄 Migration & Implementation Progress

> [!IMPORTANT]
> **Audit du 2026-10-19.** Les phases 3 à 6 étaient marquées « Completed », mais le moteur ne fonctionnait pas
> en conditions réelles. Les phases ont été reprises, corrigées et **validées en exécution réelle**. Les
> corrections sont commitées dans `~/Projects/calix` (branche `main` pour le moteur, `feature/calix-branding`
> pour l'identité visuelle).

```
Phase 0 (Completed)  : Research CachyOS (Snapper vérifié, alignement Zig upstream vérifié)
                        ⚠️ Correction : snap-pac N'EST PAS installé → pas de snapshots pacman automatiques
Phase 1 (Completed)  : Master Architecture Plan & Blueprint
Phase 2 (Completed)  : Repo renamed ~/Projects/shelly-gpui → ~/Projects/calix (history preserved + symlink)
Phase 3 (Completed)  : State synchronization engine (calix-sync) + AppImage auto-discovery
                        → 16 AppImages registered into both Shelly & Calix
Phase 4 (Completed)  : Multi-ecosystem indexing (Cargo crates in ~/.cargo/bin, NPM globals in /usr/lib)
Phase 5 (Completed)  : Calix.Release prober & updater (GitHub/GitLab/Gitea)
                        → 🔧 CORRIGÉ : `import glob` manquant (NameError après le build), comparaison de
                          versions lexicographique, GitLab/Gitea sondés via l'API GitHub.
                          Release réelle : artcraft-launcher-bin 0.4.1 → 0.5.1 (snapshot snapper #2222,
                          checksum upstream conforme, pkexec pacman -U, version vérifiée après install).
Phase 6 (Completed)  : Unified CLI wrapper + systemd units
                        → 🔧 CORRIGÉ : la CLI vivait hors du dépôt ; le timer était `disabled`.
                          `scripts/calix` est maintenant versionné (symlinké dans ~/.local/bin) et
                          `calix-sync.timer` est `enabled` + `active` (cadence 4 h).
Phase 7a (Completed) : Calix branding of the Rust GPUI frontend
                        → crate `shelly-gpui` → `calix-gpui`, app_id/socket/class renommés, 15 fichiers
                          de surface mis à jour, PKGBUILD-calix, 178 tests verts, lancement Wayland natif
                          vérifié en direct (evidence/runs/calix-branding-20261019-154900/).
Phase 7b (Completed) : Vue multi-source dans l'UI GPUI
                        → 30 paquets invisibles sont maintenant listés : 20 crates Cargo, 9 paquets NPM
                          globaux, 1 paquet suivi par le backend de releases. La vue « Installed » passe
                          de 1876 à 1906 paquets. Trois défauts trouvés en vérification live et corrigés
                          (voir §0). 190 tests verts.
Phase 8 (Next)       : Transition from Shelly to Calix (pacman -R shelly, install calix-bin)
                        ⚠️ Prérequis : le backend Zig doit d'abord passer de `shelly` à `calix`.
```

---

## 🚨 Problèmes Résolus Live

### 0. Le moteur Calix ne fonctionnait pas (RÉSOLU ✓ — audit du 2026-10-19)
- **Constat** : les phases 3→6 étaient déclarées terminées, mais le chemin critique était mort.
  - `calix update` levait `NameError: name 'glob' is not defined` **après** un `makepkg` réussi — le
    PKGBUILD était muté, le paquet compilé, et l'installation `pkexec` n'était jamais atteinte.
  - La comparaison de versions était lexicographique : `0.5.10` était classé **en dessous** de `0.5.9`.
  - `parse_repo_target` détectait GitLab/Gitea mais `probe_github` était toujours appelé → une URL
    GitLab était silencieusement sondée sur l'API GitHub.
  - Le raccourci `owner/repo` était analysé comme `host/path` et rejeté.
  - `sync_releases()` réécrivait `releases.json` depuis un littéral codé en dur → **effaçait la version
    installée** enregistrée par une mise à jour réussie.
  - La CLI vivait uniquement dans `~/.local/bin` (hors dépôt), et `calix-sync.timer` était `disabled` :
    « la synchronisation sans action manuelle » ne tournait pas.
- **Solution** : `scripts/calix_core.py` (primitives partagées et testables) + réécriture de
  `calix-release.py` et `calix-sync.py` ; `scripts/calix` versionné dans le dépôt ; unités systemd
  versionnées dans `packaging/systemd/`.
- **Preuve** : 27 tests unitaires (`tests/calix/test_calix.py`) dont un contrôle statique de noms non
  définis qui échoue exactement sur la classe de défaut qui avait cassé le chemin de mise à jour.
  Release réelle `artcraft-launcher-bin 0.4.1 → 0.5.1` exécutée de bout en bout.

### 1. AppImages non détectées par Shelly (RÉSOLU ✓)
- **Cause identifiée** : Shelly ne scannait pas `~/Applications/` dynamiquement. Il ne consultait que `~/.config/shelly/appimage-metadata-v2.db` qui ne contenait qu'une seule entrée (Copilot).
- **Solution appliquée** : `calix sync` a scanné `~/Applications/` et `~/.local/share/applications/`, extrait versions/icônes/tailles, et alimenté la DB.
- **Preuve** : `shelly list appimage` affiche désormais les **16 AppImages** (CADCraft, DeckCraft, DesignCraft, effectcraft, FigCraft, FilmCraft, GridCraft, LightCraft, OpenWork, PdfCraft, PhotoCraft, SoundCraft, Open CAD Studio/tty7, VectorCraft, WordCraft, Copilot).

### 2. Artcraft Launcher auto-update impossible (RÉSOLU ✓)
- **Cause identifiée** : Le launcher Artcraft désactive son auto-update quand installé sous forme de paquet système Arch/PKGBUILD.
- **Solution appliquée** : `calix probe storytold/craft-launcher` et `calix check` sondent l'API GitHub ; `calix update artcraft-launcher` compile et installe via `pkexec`.
- **Preuve live** : **0.4.1 → 0.5.1** installé et vérifié (`pacman -Q` = `0.5.1-1`, `artcraft-launcher --version` = `0.5.1`).
  Snapshot de sécurité `snapper #2222` créé avant la transaction (snap-pac absent), SHA256 du tarball
  conforme au manifeste amont, `calix verify artcraft-launcher` repasse au vert.

### 3. Désynchronisation multi-canaux (RÉSOLU ✓)
- **Solution appliquée** : `calix list all` agrège instantanément pacman/AUR (1875 paquets), AppImages (16), Cargo (20 crates dont les daemons gnosix), et NPM (9 paquets). State persisté dans `~/.local/state/calix/status.json`.

### 4. Shelly-GPUI (archive) vs Shelly officiel
- **Action** : Shelly-GPUI devient Calix. Shelly officiel sera désinstallé après la Phase 8.
- **Valeur de shelly-gpui** : code GPUI, architecture Zig CLI, backends ALPM/AUR/AppImage, research GPUI, tests Flatpak backend — tout ça est capitalisé dans Calix.
- **Avancement Phase 7a** : l'interface Rust GPUI porte désormais l'identité Calix (`calix-gpui`), sans
  collision de socket avec Shelly, et se lance réellement en Wayland natif.

### 5. Identité de produit incohérente (RÉSOLU ✓)
- **Constat** : `calix gui` ne lançait pas Calix — le binaire GPUI s'appelait `shelly-gpui` et `calix gui`
  retombait sur `/usr/bin/shelly` (le produit Shelly, pas Calix).
- **Solution** : crate, binaire, app_id Wayland, socket de contrôle, icônes, desktop entry et politique
  polkit renommés ; `calix gui` résout désormais le vrai binaire (y compris via un `target-dir` cargo
  redirigé) et ne retombe plus que de façon explicite et signalée.

### 6. Les écosystèmes Calix étaient invisibles dans l'UI (RÉSOLU ✓ — Phase 7b)
- **Constat** : le moteur Calix inventoriait 20 crates Cargo, 9 paquets NPM globaux et 1 paquet suivi
  par le backend de releases, mais le frontend GPUI n'affichait que ce que le CLI Shelly rapporte :
  30 paquets installés étaient invisibles.
- **Défauts trouvés en vérification live** (les tests ne les attrapaient pas) :
  1. La fusion de l'inventaire n'était câblée que sur le chemin de rechargement, pas sur le chargement
     initial → la vue affichait encore 1876 paquets au lieu de 1906.
  2. `PackageKey::from_unified` classait toute source inconnue comme ALPM → une crate Cargo héritait
     de la clé ALPM et récupérait les détails ALPM d'un paquet homonyme (l'AUR `cargo-edit`), affichant
     une URL amont et un motif d'installation qui ne concernaient pas la crate inspectée.
  3. `is_alpm` reposait sur un repli « source inconnue = ALPM » : une crate Cargo héritait des
     capacités ALPM (dépendances, build, suppression).
- **Solution** : `backend/calix_state.rs` (lecture à tolérance de panne), `merge_installed()` comme
  point de fusion unique, `PackageSourceKind::Cargo/Npm/Release`, et une sémantique dédiée aux
  écosystèmes Calix (ni installation, ni suppression, ni résolution de dépendances).
- **Preuve** : la vue « Installed » affiche **1906 paquets** ; les inspecteurs cargo / release / npm
  affichent chacun leurs propres métadonnées. 190 tests verts, avec un test de régression pour chacun
  des trois défauts (`evidence/runs/calix-multisource-20261019-161900/`).

### 7. Aucun filet de sécurité sur les transactions pacman (RÉSOLU ✓)
- **Constat** : le plan affirmait que `snap-pac` créait des snapshots pré/post sur chaque transaction
  pacman. Le hook n'était **pas installé** — aucune transaction n'était protégée.
- **Solution** : `snap-pac` installé depuis `extra` (3.0.1-3) ; le hook post-transaction a créé le
  snapshot **#2224** dès l'installation. Le filet automatique est désormais réel.

---

## 📁 Structure Repo Cible (post-renommage)

```
~/Projects/calix/          ← renommé depuis shelly-gpui
├── AGENTS.md
├── PKGBUILD               ← adapté pour calix-bin
├── calix.cli.zig/         ← renommé depuis Shelly.Cli.Zig
│   ├── src/
│   │   ├── main.zig
│   │   ├── backend/
│   │   │   ├── alpm.zig
│   │   │   ├── aur.zig
│   │   │   ├── appimage.zig
│   │   │   ├── flatpak.zig
│   │   │   ├── release.zig     ← NOUVEAU
│   │   │   ├── npm.zig         ← NOUVEAU
│   │   │   ├── pip.zig         ← NOUVEAU
│   │   │   ├── cargo.zig       ← NOUVEAU
│   │   │   └── brew.zig        ← NOUVEAU
│   │   ├── db/
│   │   │   └── surrealdb.zig   ← NOUVEAU (remplace state JSON)
│   │   └── daemon/
│   │       └── sync.zig        ← NOUVEAU
│   └── build.zig
├── calix.ui.gpui/         ← renommé depuis Shelly.Ui.Gpui
├── calix.tui/             ← renommé depuis Shelly.Tui (étendu pour agents)
├── calix.notifications/   ← renommé depuis Shelly.Notifications.Zig
├── docs/
│   ├── calix-architecture.md   ← ce document (version formelle)
│   └── migration-from-shelly.md
└── evidence/              ← logs de tests (conforme AGENTS.md)
```

---

## ⏳ Prochaines étapes immédiates

> État réel au 2026-10-19 (audit effectué contre le système, pas contre le plan).

**Déjà fait** ✔ — research CachyOS, renommage du repo, `PKGBUILD-calix`, moteur `calix-release` avec
dispatch GitHub/GitLab/Gitea, CLI unifiée versionnée, `calix-sync.service`/`.timer` installés **et
activés**, branding Calix du frontend GPUI (Phase 7a), vue multi-source Cargo/NPM/releases (Phase 7b),
`snap-pac` installé.

**À faire, par ordre de valeur :**

1. **[ ] Renommer le backend Zig** `Shelly.Cli.Zig` → `calix.cli.zig` (binaire `shelly` → `calix`).
   C'est le prérequis de la Phase 8 : le frontend parle encore au CLI via `SHELLY_BIN`.
   Le codebase Zig fait ~69 000 lignes avec 408 références à `shelly` — prévoir des étapes (identifiants
   d'action polkit, chemins XDG, protocole IPC, noms de snapshots de test) plutôt qu'un remplacement global.
2. **[ ] Brancher les actions d'installation/suppression sur les écosystèmes Calix.** Les paquets Cargo,
   NPM et releases sont listés mais aucune action n'est proposée : il faut un chemin d'exécution pour
   `cargo install --locked`, `npm i -g`, et la reconstruction PKGBUILD du backend de releases.
3. **[ ] Étendre la recherche aux écosystèmes Calix.** Le rail de recherche ne couvre encore que
   ALPM/AUR/Flatpak/AppImage ; l'inventaire Calix n'est pas interrogeable.
4. **[ ] Décider du stockage.** L'architecture prévoit SurrealDB embedded ; l'implémentation actuelle
   utilise des JSON (`~/.local/state/calix/*.json`) avec un registre de suivi dans
   `~/.config/calix/releases.json`. SurrealDB reste un objectif, pas une obligation : le JSON est
   versionnable, diffable et suffisant à l'échelle actuelle.
5. **[ ] Phase 8 — Transition** : `pacman -R shelly` + installation de `calix-bin`. Ne pas le faire
   avant l'étape 1 : le frontend dépend encore du CLI Shelly.
6. **[ ] Renommer le dépôt GitHub** `tension-atoi/shelly-gpui` → `tension-atoi/calix` (action manuelle,
   hors de portée de l'agent).

---

> [!TIP]
> Le `/goal` slash command peut être utilisé pour lancer une exécution longue et non-stop de toutes les phases.

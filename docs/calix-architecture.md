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
Tranche 0 (Completed): Deux bugs vivants du backend Zig corrigés
                        → ordering des tags de release (shared/release_version.zig) et fin de
                          l'escalade silencieuse par sudo (findElevator → pkexec).
                          Preuve de régression : avec la logique d'origine, exactement 4 des nouveaux
                          tests échouent et tous les tests préexistants passent.
Tranche 1 (Next)     : Absorption du moteur Python dans le Zig
                        → backends Cargo et NPM, discovery ~/Applications dans le manager appimage
                          Zig, registre de releases, vérification de provenance amont, flux
                          d'application, helper root + polkit, inventaire unifié versionné,
                          calix-daemon, calix CLI, UI sur daemon, suppression du Python.
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

### 9. La copie AUR du PKGBUILD avait divergé (RÉSOLU ✓)
- **Constat** : `packaging/aur/PKGBUILD` était une **copie périmée** de l'ancien `PKGBUILD-gpui`. Elle
  installait encore `shelly-gpui`, l'ancienne desktop entry `com.shellyorg.shelly-gpui.desktop` et
  l'ancienne icône — c'est-à-dire que le branding de la Phase 7a n'avait jamais atteint la copie
  publiée sur l'AUR. Le README pointait en outre vers `PKGBUILD-gpui`, un fichier supprimé : lien mort.
- **Solution** : `packaging/aur/PKGBUILD` est désormais un **lien symbolique** vers `PKGBUILD-calix`.
  Une seule définition de paquet, donc aucune dérive possible — la copie AUR ne peut plus.publishier
  autre chose que l'in-tree.
- **Preuve** : `packaging/aur/PKGBUILD` et `PKGBUILD-calix` sont désormais le même fichier, par
  construction.

### 8. Deux bugs vivants dans le backend Zig (RÉSOLU ✓ — tranche 0)
- **Défaut 1 — ordering des tags de release** : `updateIsAvailable()` ne comparait les versions que si
  **les deux** étaient purement numériques pointés. Toute autre paire retombait sur « toute différence
  est une mise à jour », donc **les rétrogradations étaient proposées comme mises à jour** : une 1.7.4
  stable installée avec la 1.7.4-rc1 du forge, ou une 2.0.0-rc1 installée avec la 1.9.0 du forge.
  Corrigé par `Shelly.PackageManager/src/shared/release_version.zig` : comparaison du core numérique +
  précédence de prerelease SemVer, et `null` pour les tags non ordonnables (ETags, jetons de date) afin
  de préserver le repli historique. `isNumericDottedVersion` et `compareNumericVersions` supprimés (morts).
- **Défaut 2 — escalade silencieuse par sudo** : `findElevator()` scannait le `PATH` pour `doas`, `sudo`,
  `run0` — dans cet ordre — et retombait sur `"sudo"`. `AGENTS.md` interdit `sudo`, `su`, `doas` et tout
  équivalent ; l'élévation doit passer par `pkexec`/polkit. **La violation était invisible** parce que
  l'UI Rust injectait `SHELLY_ELEVATOR=pkexec` dans l'environnement de l'enfant : la politique était
  masquée par l'appelant, le pire état possible. Le scan implicite est supprimé ; `SHELLY_ELEVATOR`
  reste prioritaire comme choix explicite de l'opérateur (y compris `run0`) ; le défaut est `pkexec`.
  `isOnPath` supprimé (mort). Aucun changement de comportement sur cette machine : la détection
  automatique de `run0` était déjà inatteignable, `sudo` le masquant dans l'ancienne précédence.
- **Preuve** : avec la logique d'origine restaurée à l'identique, **exactement 4 des nouveaux tests
  échouent** et tous les tests préexistants passent — le changement est strictement additif.
  PackageManager 1323/1327 (4 skipped, +15 tests), CLI 376/376.
  (`evidence/runs/calix-tranche0-20261019-172500/`)

---

## 🧭 Décisions d'architecture (2026-10-19, session de cadrage)

Ces décisions remplacent les hypothèses du plan initial. Elles ont été prises contre l'état réel du
système, et plusieurs d'entre elles **invalident** des affirmations plus anciennes de ce document.

| # | Décision | Conséquence directe |
|---|----------|---------------------|
| 1 | **Zig est la destination.** Le moteur Python est absorbé ; le daemon ne l'exécute jamais, même temporairement | Un « pont temporaire » sans échéance devient permanent. Le plan avait déjà fait ce coup avec Shelly. |
| 2 | **L'UI ne lit plus aucun fichier d'état.** Le daemon est le point d'entrée unique, y compris pour les actions | La Phase 7b (lecture directe de `~/.local/state/calix/*.json`) est un prototype à re-héberger, pas un acquis. |
| 3 | **Daemon non privilégié.** La frontière de privilège est un helper root-owned à capacité étroite ; l'artefact est validé (checksum amont + provenance) avant toute installation root | Un daemon qui s'élève transforme un bug en compromission root. |
| 4 | **Le probing Zig est la référence.** Le correctif prerelease entre en premier dans le Zig | Le Zig a déjà 6 forges contre 3 côté Python, et `aur/version.zig` applique les règles de version Arch. |
| 5 | **Le défaut `sudo` de `findElevator` est corrigé dans le Zig**, plus contourné par l'appelant | Corrigé — voir §8. |
| 6 | **Tranche 1 = absorption** (daemon + inventaire unifié, backends Cargo/NPM inclus). Le surveillant de releases vient ensuite | Sans les backends Cargo/NPM dans le Zig, la vue multi-source régresserait de 1906 à 1876 paquets. |
| 7 | **Le daemon normalise**, l'UI ne fait que rendre. Schéma versionné | Sans version de schéma, l'UI rendra silencieusement faux contre un daemon plus récent — la classe de bug de l'homonymie. |
| 8 | **L'inventaire est dérivé, pas de l'état.** Registre + « déjà notifié » en JSON dans le répertoire d'état | SurrealDB est mort : pas de SDK Zig, « embedded » inexistant en Zig, jamais écrit, et l'inventaire n'est pas de l'état. |
| 9 | **Le daemon a son journal opérationnel dans son répertoire d'état**, pas dans le dépôt | Un timer permanent n'écrit pas dans l'arbre git. La preuve (`evidence/`) est un contrat de développement. |
| 10 | **État final : `calix` + `calix-daemon`**, une bibliothèque de backends partagée ; la CLI fonctionne en autonome *et* contre le daemon | Un one-shot ne doit pas avoir besoin de réveiller un démon. |
| 11 | **Le renommage du backend avance avec le daemon**, pas « après la preuve » | Le renommage n'est plus bloqué que par le couplage de l'UI au `shelly` installé — et ce couplage disparaît quand le daemon devient le point d'entrée. |

> [!IMPORTANT]
> **Ce que le plan initial affirmait à tort, et qui est maintenant faux.** Le moteur Python n'était pas
> « le backend » : c'était un doublon partiel d'un backend Zig de 74 000 lignes, dont ~60 % de la
> logique existait déjà côté Zig — y compris le probing de releases, en mieux. Le daemon n'existait pas,
> et la « base SurrealDB embedded » n'avait jamais été écrite. Le plan affirmait aussi que `snap-pac`
> créait des snapshots automatiques sur chaque transaction pacman : le hook n'était pas installé.

---

## ⏳ Prochaines étapes immédiates

> État réel au 2026-10-19 (audit effectué contre le système, pas contre le plan).

**Déjà fait** ✔ — research CachyOS, renommage du repo, `PKGBUILD-calix`, moteur `calix-release` avec
dispatch GitHub/GitLab/Gitea, CLI unifiée versionnée, `calix-sync.service`/`.timer` installés **et
activés**, branding Calix du frontend GPUI (Phase 7a), vue multi-source Cargo/NPM/releases (Phase 7b),
`snap-pac` installé, **tranche 0** (ordering des tags de release + fin de l'escalade `sudo`).

### Tranche 1 — l'absorption (le prochain chantier)

Le moteur Python (2013 lignes) est absorbé dans le Zig. Inventaire exact de ce qui est **déjà** dans le
Zig et ne doit donc pas être porté : client HTTP, E/S JSON, parsing d'arguments, rendu de tableaux,
lancement de processus, classification des forges et des assets, et le probing lui-même.

Travail réellement nouveau, en Zig :

1. **Backend Cargo** — lecture de `~/.cargo/.crates2.json`.
2. **Backend NPM** — `npm list -g`.
3. **Discovery `~/Applications`** — à intégrer au manager appimage Zig existant. Supprime la course à
   deux propriétaires sur `appimage-metadata-v2.db` (le Python y écrit aujourd'hui).
4. **Registre de releases** + état (JSON, répertoire d'état du daemon).
5. **Vérification de provenance amont** — récupérer le `SHA256SUMS.txt` du forge et le comparer à
   l'artefact téléchargé. *Cette capacité n'existe pas dans le Zig aujourd'hui* : `checksums.zig`
   vérifie l'artefact contre les sommes du PKGBUILD, ce qui est circulaire quand `updpkgsums` régénère
   ces sommes depuis le fichier qu'il vient de télécharger.
6. **Flux d'application** — bump PKGBUILD → vérif amont → `makepkg` → helper root → vérif → rollback.
7. **Helper root** à capacité étroite + règle polkit dédiée.
8. **Agrégation de l'inventaire** + document versionné (schéma versionné, test de compatibilité Zig↔Rust).
9. **`calix-daemon`** — socket, scheduler, journal opérationnel, persistance du « déjà notifié ».
10. **`calix` CLI** en Zig, partageant la bibliothèque, fonctionnant en autonome *et* contre le daemon.
11. **L'UI passe par le daemon** — la lecture directe de `~/.local/state/calix/*.json` (Phase 7b)
    disparaît ; le modèle `UnifiedPackage` Rust devient un pur rendu.
12. **Suppression du moteur Python.**

### Après la tranche 1

- **Tranche 2 — le surveillant de releases** : le daemon interroge les paquets suivis en continu et
  notifie. C'est la capability qu'aucun CLI invoqué à la demande ne peut porter.
- **Tranche 3 — l'application automatique** : helper root, fenêtre de mise à jour, notification
  Wayland avant application, rattrapage après redémarrage.
- **Tranche 4 — inotify** : détection live des changements (`~/Applications`, `~/.cargo/bin`,
  `/usr/lib/node_modules`).
- **Actions d'installation/suppression** demandées par l'UI pour les écosystèmes Calix
  (`cargo install --locked`, `npm i -g`, reconstruction PKGBUILD de release).
- **Recherche étendue** aux écosystèmes Calix — le rail de recherche ne couvre encore qu'ALPM/AUR/Flatpak/AppImage.
- **Phase 8 — transition** : le frontend ne dépendant plus du `shelly` installé, le backend peut être
  renommé `calix` ; le `shelly` installé peut alors être retiré.
- **[x] Renommer le dépôt GitHub** `tension-atoi/shelly-gpui` → `tension-atoi/calix` — fait par
  l'opérateur le 2026-10-19 ; l'historique est préservé et le remote local pointe désormais sur la
  nouvelle URL. Toutes les références vivantes du dépôt (PKGBUILD, README, CONTRIBUTING, SECURITY,
  templates d'issue, contrats de slice) ont été mises à jour. Les rapports d'acceptation signés et
  leurs captures de sortie brute n'ont volontairement pas été réécrits : ce sont des enregistrements
  datés, assortis de hashes de commit, et le lien GitHub redirige de toute façon.

---

> [!TIP]
> Le `/goal` slash command peut être utilisé pour lancer une exécution longue et non-stop de toutes les phases.

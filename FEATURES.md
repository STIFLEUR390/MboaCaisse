# MboaCaisse — Fonctionnalités Tauri (par ordre d'implémentation)

> App interne Rust/Tauri pour bar/restaurant/épicerie.
> **Devise** : FCFA | **Réseau** : LAN local, sans internet obligatoire.
> **Stack** : Tauri 2 + Axum embarqué + libsql (Turso) + Nuxt 4.

---

## Priorité P0 — Socle (faire fonctionner)

### P0.1 Serveur HTTP Axum embarqué

Le frontend Nuxt est **servi directement par Axum** (fichiers statics + API). Pas de plugin localhost — Axum fait office de serveur web complet.

| Feature | Detail |
|---------|--------|
| **Axum dans Tauri** | Lancé dans un `tokio::spawn` au `setup()` de Tauri |
| **Sert le frontend** | Les fichiers `dist/` sont servis statiquement par Axum |
| **Sert l'API** | Toutes les routes `/api/*` gérées par Axum |
| **Écoute `0.0.0.0:PORT`** | Accessible depuis tout le LAN |
| **Fenêtre native** | Charge `http://localhost:PORT` (via `devUrl` en dev, Axum en prod) |
| **Clients LAN** | Navigateurs pointant vers `http://IP_SERVEUR:PORT` |
| **CSP désactivé** | `"csp": null` dans `tauri.conf.json` |
| **Détection IP** | Affiche l'URL à partager au démarrage |

**Implémentation**
- Ajouter `axum`, `tokio`, `tower-http` dans `Cargo.toml`
- Lancer Axum dans `setup()` via `tokio::spawn`
- Axum sert : `tower-http::services::ServeDir::new("dist")` pour le frontend
- Axum gère : routes `/api/*` pour la logique métier
- `devUrl` dans `tauri.conf.json` pointe vers `http://localhost:PORT` (port fixe connu)
- Frontend Nuxt appelle Axum via `fetch("/api/...")`

**Avantage** : Pas de plugin Tauri supplémentaire. Architecture plus simple : un seul serveur HTTP (Axum) qui fait tout.

#### Découverte réseau — `mboacaisse.local`

| Feature | Detail |
|---------|--------|
| **mDNS/Avahi** | L'app s'annonce sur le LAN comme `mboacaisse.local` |
| **Aucun reverse proxy** | Pas de nginx/caddy — Axum écoute direct `0.0.0.0:PORT` |
| **Clients** | Navigateurs LAN → `http://mboacaisse.local:PORT` |
| **Linux** | Avahi + `libavahi-client` ou appel système `avahi-publish-service` |
| **macOS** | Bonjour natif (via `dns-sd` ou Bonjour SDK) |
| **Windows** | WS-Discovery ou mDNS via `dnssd` |

Les postes clients n'ont pas besoin de connaître l'IP. L'URL `http://mboacaisse.local` fonctionne sur tout le LAN sans configuration DNS.

**Implémentation Rust (Linux via Avahi)**

```rust
use std::process::Command;

// Au démarrage d'Axum
fn publish_mdns(port: u16) {
    Command::new("avahi-publish-service")
        .args([
            "MboaCaisse",
            "_http._tcp",
            &port.to_string(),
            "path=/",
        ])
        .spawn()
        .ok();
}
```

Alternative sans binaire externe : crate `mdns-sd` pour mDNS pur Rust (multi-plateforme, pas de dépendance Avahi).

### P0.2 Base de données libsql embarquée

| Feature | Detail |
|---------|--------|
| **libsql (Turso)** | SQLite-compatible, async-first, writes concurrents |
| **Fichier unique** | `mboacaisse.db` dans `$APP_DATA_DIR` |
| **Connexion Rust** | `libsql::Database::open()` dans le contexte Tauri |

### P0.3 Authentification & sessions

| Feature | Detail |
|---------|--------|
| **Login local** | Email + mot de passe (argon2), pas de SSO/OAuth |
| **Session JWT** | Token stocké en cookie HTTP-only, vérifié côté Axum |
| **4 rôles** | `admin`, `caissier`, `vendeur`, `gestionnaire_stock` |
| **Middleware Axum** | Guard par rôle sur chaque route `/api/*` |

### P0.5 Structure du projet Rust

Organisation par domaine métier (pas par couche technique) :

```
src-tauri/
├── src/
│   ├── main.rs              # Entry point Tauri
│   ├── lib.rs               # Builder Tauri, plugins, setup
│   ├── server.rs            # Lancement Axum + shutdown handle
│   ├── mdns.rs              # Publication mDNS (mboacaisse.local)
│   ├── db/
│   │   ├── mod.rs
│   │   ├── migrations.rs    # Migrations schema embarquées
│   │   └── seed.rs          # Données de démo optionnelles
│   ├── api/
│   │   ├── mod.rs           # Router Axum général
│   │   ├── auth.rs          # Login, logout, middleware JWT
│   │   ├── products.rs      # CRUD produits
│   │   ├── categories.rs    # CRUD catégories
│   │   ├── orders.rs        # Commandes (cycle de vie)
│   │   ├── payments.rs      # Encaissement, moyens paiement
│   │   ├── balance.rs       # Solde client, transactions
│   │   ├── stock.rs         # Mouvements, alertes
│   │   ├── suppliers.rs     # Fournisseurs
│   │   ├── reports.rs       # Rapports, stats
│   │   ├── employees.rs     # Gestion employés (admin)
│   │   └── settings.rs      # Paramètres système
│   ├── domain/
│   │   ├── mod.rs
│   │   ├── user.rs          # Struct User, role enum
│   │   ├── product.rs       # Struct Product
│   │   ├── order.rs         # Struct Order, OrderStatus
│   │   ├── payment.rs       # PaymentMethod enum
│   │   └── ...
│   └── license/
│       ├── mod.rs
│       ├── verify.rs        # Vérification signature Ed25519
│       └── entitlements.rs  # Feature gating
└── Cargo.toml
```

**Règle** : `domain/` contient les structures pures (pas de dépendance Tauri ou Axum). `api/` contient les handlers. `db/` contient l'accès aux données.

### P0.6 Fenêtre native & Tray

| Feature | Status | Detail |
|---------|--------|--------|
| **Fenêtre principale** | ✅ Fait | 1366×768, min 375×812, redimensionnable |
| **Tray icon** | ✅ Fait | Menu Quit, clic gauche ouvre menu |
| **Menu natif** | ✅ Fait | Quit item |
| **Fenêtre secondaire** | 🔧 À faire | Pour afficheur client (voir P2.2) |

---

## Priorité P1 — Modules métier (MVP utilisable)

### P1.1 Plugin `store` + `@tauri-store/pinia`

| Feature | Status | Detail |
|---------|--------|--------|
| **Plugin store** | ✅ Fait | `tauri_plugin_store::Builder::new().build()` |
| **Pinia bridge** | 🔧 À faire | Synchroniser stores Pinia ↔ Tauri store (frontend) |
| **Cache local** | 🔧 À faire | Préférences utilisateur, panier en cours, état UI |

### P1.2 Backup BDD — téléchargeable par l'admin

#### Sauvegarde côté Rust

| Feature | Detail |
|---------|--------|
| **Fichier** | `mboacaisse.db` copié + archivé |
| **Format** | ZIP contenant le `.db` + metadata (date, version, taille) |
| **Déclenchement** | Manuel (bouton admin) + automatique (tâche tokio toutes les X heures) |
| **Rotation** | Garde les N dernières sauvegardes (ex: 30) |
| **Avant MAJ** | Sauvegarde auto avant installation mise à jour |

Le ZIP est généré côté Rust (crate `zip`) et stocké dans `$APP_DATA_DIR/backups/`.

```rust
use std::fs::File;
use zip::ZipWriter;

fn create_backup(db_path: &Path, backup_dir: &Path, app_version: &str) -> Result<PathBuf> {
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let zip_path = backup_dir.join(format!("mboacaisse_{timestamp}.zip"));

    let file = File::create(&zip_path)?;
    let mut zip = ZipWriter::new(file);

    // Ajouter le fichier DB
    zip.start_file("mboacaisse.db", ...)?;
    let mut db_file = File::open(db_path)?;
    io::copy(&mut db_file, &mut zip)?;

    // Ajouter metadata
    zip.start_file("backup.json", ...)?;
    let meta = serde_json::json!({
        "timestamp": timestamp,
        "version": app_version,
        "db_size_bytes": db_file_size
    });
    zip.write_all(serde_json::to_string_pretty(&meta).as_bytes())?;

    zip.finish()?;
    Ok(zip_path)
}
```

#### Téléchargement

| Méthode | Detail |
|---------|--------|
| **API Axum** | Route `GET /api/backups/{filename}` → sert le fichier ZIP |
| **Admin only** | Vérification rôle admin avant téléchargement |
| **Listing** | `GET /api/backups` → liste des sauvegardes disponibles |
| **Frontend** | Interface admin : liste des backups + bouton télécharger |

```rust
// Route Axum
async fn download_backup(
    Path(filename): Path<String>,
    Extension(auth): Extension<AdminAuth>,
) -> impl IntoResponse {
    let path = backup_dir.join(&filename);
    let file = tokio::fs::read(path).await?;
    [
        (header::CONTENT_TYPE, "application/zip"),
        (header::CONTENT_DISPOSITION, &format!("attachment; filename=\"{filename}\"")),
    ]
    .into_response(file)
}
```

#### Restauration

| Feature | Detail |
|---------|--------|
| **Upload ZIP** | Admin upload un fichier `.zip` depuis l'interface |
| **Extraction** | Rust extrait `mboacaisse.db` du ZIP (vérifie `backup.json`) |
| **Remplacement** | Copie le fichier extrait à l'emplacement de la BDD active |
| **Redémarrage requis** | L'app doit redémarrer après restauration |
| **Double confirmation** | Modal : "Remplacer la base actuelle ? Cette action est irréversible." |
| **Backup auto avant restore** | L'état actuel est sauvegardé avant la restauration (filet de sécurité) |

```rust
// Route Axum — upload + restore
async fn restore_backup(
    Extension(auth): Extension<AdminAuth>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    while let Some(field) = multipart.next_field().await.unwrap() {
        if field.name() == Some("backup") {
            let bytes = field.bytes().await.unwrap();
            let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes))?;

            // Backup auto de l'état actuel avant restore
            create_backup(&db_path, &backup_dir, app_version)?;

            // Extraire le fichier DB
            let mut db_file = archive.by_name("mboacaisse.db")?;
            let mut dest = std::fs::File::create(&db_path)?;
            std::io::copy(&mut db_file, &mut dest)?;

            // Redémarrage
            app_handle.restart(); // ou exit + service systemd
        }
    }
}
```

### P1.3 Plugin `notification`

| Feature | Status | Detail |
|---------|--------|--------|
| **Plugin notification** | ✅ Fait | `tauri_plugin_notification::init()` |
| **Permissions** | ✅ Fait | `notification:default` |
| **Nouvelle commande** | 🔧 À faire | Notification au caissier quand vendeur soumet |
| **Stock bas** | 🔧 À faire | Alerte seuil configurable |
| **Demande clôture** | 🔧 À faire | Vendeur demande paiement → notif caissier |

### P1.4 Plugin `shell`

| Feature | Status | Detail |
|---------|--------|--------|
| **Plugin shell** | ✅ Fait | `tauri_plugin_shell::init()` |
| **Permissions** | ✅ Fait | `shell:allow-open`, `shell:allow-execute` |
| **Ouvrir dossier** | 🔧 À faire | Ouvrir le dossier de backups dans le gestionnaire de fichiers |
| **Impression** | 🔧 À faire | Envoyer fichier vers imprimante via commande système |

---

## Priorité P2 — Fonctionnalités avancées Tauri

### P2.1 Impression thermique native

| Feature | Detail |
|---------|--------|
| **ESC/POS** | Génération buffer binaire via `escpos-rs` ou `printpdf` |
| **USB natif** | Écriture sur `/dev/usb/lp*` (Linux) via Rust |
| **Réseau** | Impression sur imprimante TCP/IP (port 9100) |
| **Formats** | 58mm (ticket caisse) et 80mm (cuisine/bar) |
| **Auto-impression** | Après validation caisse, envoi direct à l'imprimante |

```
Commande validée (caissier)
  → Rust génère buffer ESC/POS (logo, articles, total, remerciements)
  → Écrit sur /dev/usb/lp0 (ou socket TCP:9100)
  → Ticket imprimé, pas de popup navigateur
```

**Avantage Tauri** : L'impression native depuis Rust contourne les limitations navigateur (pas de `window.print()` fragile, pas de popup bloquée, contrôle total du format).

### P2.2 Afficheur client (seconde fenêtre)

| Feature | Detail |
|---------|--------|
| **Seconde fenêtre Tauri** | `WebviewWindow::builder("secondary", ...)` dans `setup()` |
| **URL** | `http://localhost:PORT/afficheur` (servi par Axum) |
| **Plein écran** | Pas de barre d'adresse, pas de navigation |
| **Mise à jour** | WebSocket ou polling depuis Axum |
| **Contenu** | Logo + montant total + articles en cours |
| **Mode veille** | Si aucune commande active, logo + message |

**Implémentation**
```rust
// Dans setup() de lib.rs
let webview_window = tauri::WebviewWindowBuilder::new(
    app,
    "secondary",
    tauri::WebviewUrl::External("http://localhost:PORT/afficheur".parse().unwrap()),
)
.title("Afficheur client")
.fullscreen(true)
.build();
```

### P2.3 Scan code-barres

| Feature | Detail |
|--------|--------|
| **Plugin** | `tauri-plugin-barcode-scanner` |
| **Mode caméra** | Scan via caméra (smartphone/tablette) |
| **Mode USB** | Scanner USB branché → émulé comme clavier |
| **Champ dédié POS** | Focalisé automatiquement, ajout instantané au panier |

### P2.4 Auto-updater

| Feature | Detail |
|---------|--------|
| **Plugin** | `tauri-plugin-updater` |
| **Source** | GitHub Releases (ou serveur dédié) |
| **Vérification** | Au démarrage + périodique |
| **Signature** | Mise à jour signée (clé privée éditeur) |
| **Backup avant MAJ** | Sauvegarde auto de la BDD avant installation |

### P2.5 Global shortcuts

| Feature | Detail |
|---------|--------|
| **Plugin** | `tauri-plugin-global-shortcut` |
| **Raccourcis caisse** | F1=scan, F2=recherche, F3=paiement, Esc=annuler |
| **Même fenêtre en fond** | Capturés même si fenêtre pas au premier plan |

### P2.6 Plugins support (autostart + log)

| Plugin | Feature | Detail |
|--------|---------|--------|
| `autostart` | Démarrage auto | Lance l'app au démarrage du PC serveur |
| `log` | Tracing | Journalisation structurée dans fichier + stdout |

### P2.7 WebSocket temps réel

| Feature | Detail |
|---------|--------|
| **Méthode** | `axum::extract::ws` (natif, pas de plugin Tauri) |
| **Usage** | Plan des tables en direct, notifications, afficheur client |
| **Clients** | Fenêtre native + tous les navigateurs LAN connectés |

---

## Priorité P3 — Packaging & distribution

### P3.1 Bundle multi-plateforme

| Plateforme | Formats | Status |
|------------|---------|--------|
| **Linux** | `.deb`, `.rpm`, `.AppImage` | 🔧 Config Tauri existante |
| **macOS** | `.dmg` | ❌ À configurer |
| **Windows** | `.msi` + `.exe` | ❌ À configurer |

**Config actuelle** (`tauri.conf.json`):
```json
{
  "bundle": {
    "active": true,
    "targets": "all",
    "linux": { "deb": {} }
  }
}
```

### P3.2 Dépendances système

| Plateforme | Paquets |
|------------|---------|
| **Debian/Ubuntu** | `libwebkit2gtk-4.1-dev`, `libsqlite3-0`, `libgtk-3-dev`, `librsvg2-dev`, `libayatana-appindicator3-dev` |
| **Fedora** | `webkit2gtk4.1-devel`, `libsqlite3x-devel`, `gtk3-devel`, `librsvg2-devel`, `libappindicator-gtk3-devel` |

---

## Priorité P4 — Système de licences

> Extrait de `docs/Tauri_mboacaise_licence.md` — système de licensing offline-first avec vérification cryptographique.

### P4.1 Architecture licensing

```
Mboa License Server (cloud)
  │  Private Key Ed25519
  │
  ▼
Licence signée (JSON + signature)
  │
  ▼
MboaCaisse (client)
  │  Public Key embarquée
  ▼
Vérification locale (offline)
```

### P4.2 Identifiants (4 niveaux)

| ID | Rôle | Exemple |
|----|------|---------|
| **Customer ID** | Identifie le client | `CUS-8A71F4` |
| **Organization ID** | Identifie le groupe | `ORG-2B19D7` |
| **License ID** | Contrat commercial | `LIC-5C29F8` |
| **Installation ID** | Installation réelle | `INS-91E2AC` |

### P4.3 Licence signée Ed25519

```json
{
  "license_id": "LIC-5C29F8",
  "customer_id": "CUS-8A71F4",
  "organization_id": "ORG-2B19D7",
  "product": "mboacaisse",
  "edition": "professional",
  "features": ["pos", "inventory", "kitchen", "reports"],
  "max_installations": 1,
  "issued_at": "2026-07-21T10:00:00Z",
  "updates_until": "2027-07-21T10:00:00Z",
  "signature": "<ed25519_sig>"
}
```

### P4.4 Entitlements (feature gating)

Au lieu de coder `edition = professional` en dur, chaque feature est un entitlement :

```rust
#[tauri::command]
fn check_feature(feature: &str, license: &License) -> bool {
    license.features.contains(&feature.to_string())
}
```

### P4.5 Modèle commercial

| Élément | Valeur |
|---------|--------|
| **Licence** | Perpétuelle |
| **Mises à jour** | Incluses 12 mois |
| **Correctifs sécurité** | Gratuits (politique support) |
| **Après 12 mois** | App continue de fonctionner. Pas d'accès aux nouvelles versions sans renouvellement ou upgrade |

---

## Résumé plugins Tauri — statut

| Plugin | Priorité | Status | Raison |
|--------|----------|--------|--------|
| `shell` | P1.4 | ✅ Plugin + permissions | Ouverture fichiers, commandes système |
| `notification` | P1.3 | ✅ Plugin + permissions | Alertes OS (commandes, stock bas) |
| `os` | P0 | ✅ Plugin + permissions | Infos plateforme |
| `fs` | P1.2 | ✅ Plugin + permissions | Export, backup, upload |
| `store` | P1.1 | ✅ Plugin + pinia bridge à faire | Cache local, préférences |
| `barcode-scanner` | P2.3 | ❌ À installer | Scan code-barres caméra |
| `updater` | P2.4 | ❌ À installer | Mise à jour automatique |
| `global-shortcut` | P2.5 | ❌ À installer | Raccourcis clavier globaux |
| `autostart` | P2.6 | ❌ À installer | Lancement au démarrage |
| `log` | P2.6 | ❌ À installer | Journalisation fichier |
| `localhost` | — | ❌ Supprimé | Remplacé par Axum qui sert tout |

---

## Conventions Tauri (existantes)

| Convention | Detail |
|------------|--------|
| **Commandes Tauri** | Une commande = une action métier, pas une requête SQL |
| **Permissions** | Déclarées dans `capabilities/main.json` |
| **Sécurité** | Commandes sensibles non exposées aux clients web (vérifier origine) |
| **Build** | `bun run tauri:build` |
| **Auto-import** | Module custom `app/modules/tauri.ts` : `useTauri<Prefix><ExportedName>` |
| **Tray** | ✅ Menu Quit. À étendre : ouvrir/fermer fenêtre, statut serveur |

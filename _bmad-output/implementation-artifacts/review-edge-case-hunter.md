# Edge Case Hunter Review — Story 1.2

## Edge Cases Identified

### HIGH: Race condition entre démarrage fenêtre et serveur Axum
**Fichier:** `lib.rs`
**Scénario:** Le serveur est spawné dans `tauri::async_runtime::spawn()` mais la fenêtre Tauri s'ouvre immédiatement après. Si le serveur n'a pas fini de binder dans les ~100ms, la fenêtre charge une page vierge ou une erreur.
**Impact:** Page blanche au premier lancement sur machine lente.
**Mitigation:** Ajouter un mécanisme d'attente (oneshot channel) de `server::start_server` vers `setup` pour confirmer que le serveur écoute avant de rendre la main.

### HIGH: resolve_dist_path() ne fonctionne pas en mode Tauri build (prod)
**Fichier:** `server.rs:99-108`
**Scénario:** En prod Tauri (`bun run tauri:build`), les fichiers sont intégrés dans le binaire. Le dossier `../dist` n'existe pas dans le contexte de l'exécutable. `dist/` non plus. Le serveur démarre mais sert des 404 sur toutes les routes UI.
**Impact:** L'interface ne charge pas en production.
**Mitigation:** Utiliser `app.path().resource_dir()` de Tauri pour trouver le bon dossier, ou un chemin absolu déterminé à la compilation.

### MEDIUM: mDNS host_ip = "0.0.0.0" peut ne pas fonctionner
**Fichier:** `mdns.rs:44`
**Scénario:** `mdns-sd` avec `"0.0.0.0"` comme host_ip. Certaines implémentations mDNS peuvent rejeter cette IP ou ne pas répondre correctement.
**Impact:** mDNS ne fonctionne pas sur certains routeurs/OS.
**Mitigation:** Résoudre l'IP locale réelle au démarrage.

### MEDIUM: backup_database() peut échouer si la BDD est verrouillée
**Fichier:** `lib.rs:188`
**Scénario:** Si une requête est en cours (pool r2d2 a une connexion active), `std::fs::copy()` échoue sur un fichier SQLite avec `database is locked`.
**Impact:** Backup perdu, warning loggué.
**Mitigation:** Utiliser `VACUUM INTO` (SQLite 3.27+) ou `backup::Backup` de rusqlite.

### MEDIUM: Le 500ms sleep dans ExitRequested est un timer magique
**Fichier:** `lib.rs:154`
**Scénario:** Le serveur peut prendre plus de 500ms à drainer (connexions longues). La backup commence avant la fin du drain.
**Impact:** Backup inconsistante + fuite de connexions.
**Mitigation:** Utiliser un canal de confirmation depuis `start_server()` pour signaler que le drain est terminé.

### LOW: Le watch channel peut avoir un délai entre send et réception
**Fichier:** `lib.rs`
**Scénario:** `watch::Sender::send()` notifie les receivers de façon asynchrone. Le `changed().await` dans le serveur peut ne pas voir immédiatement la valeur `true`.
**Impact:** Léger délai (quelques microsecondes) avant que le serveur ne commence le drain.
**Mitigation:** Utiliser `send_modify()` au lieu de `send()` pour garantir l'ordre.

### LOW: CompressionLayer non configuré
**Fichier:** `server.rs:37`
**Scénario:** `CompressionLayer::new()` utilise les valeurs par défaut (gzip, qualité 6). Pour des assets statiques servis sur LAN (<10ms), la compression ajoute de la latence CPU.
**Impact:** Légère dégradation des performances sur LAN. Acceptable en alpha.
**Mitigation:** Configurer un seuil de taille minimum.

### LOW: Pas de timeout sur le backup
**Fichier:** `lib.rs:183-192`
**Scénario:** Si le fichier est très volumineux (plusieurs Go), `std::fs::copy()` peut prendre plus de 5s.
**Impact:** L'arrêt de l'application peut être retardé.
**Mitigation:** `tokio::time::timeout()` sur l'opération de backup.

### INFO: Pas de gestion du port déjà utilisé
**Fichier:** `server.rs:55-58`
**Scénario:** Si le port 3000 est déjà utilisé par un autre processus, `TcpListener::bind` échoue, le serveur log un warning et retourne.
**Impact:** L'application démarre sans serveur HTTP. L'utilisateur ne voit pas d'erreur visible.
**Mitigation:** Scanner un range de ports (3000-3099) comme le fait le dev runner, ou afficher un message dans la fenêtre native.

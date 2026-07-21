# Système de licences — MboaCaisse

MboaCaisse est un produit commercial offline-first avec un système de licences online-capable.
Le fonctionnement quotidien du restaurant ne dépend pas d'Internet. Internet est utilisé uniquement pour l'achat, l'activation initiale, les mises à jour, et les vérifications périodiques facultatives.

---

## 1. Écosystème

```
┌──────────────────────────────────────────────────────┐
│                  ÉCOSYSTÈME MBOACAISSE               │
│                                                      │
│  1. MboaCaisse Server                                │
│     Application Tauri + Rust                         │
│     Installée chez le restaurant                     │
│                                                      │
│  2. MboaCaisse Web                                   │
│     Vue.js                                           │
│     Accessible sur le réseau local                   │
│                                                      │
│  3. Mboa License Platform                            │
│     Service cloud propriétaire                       │
│     Licences + paiements + activations               │
└──────────────────────────────────────────────────────┘
```

Internet n'est pas nécessaire au fonctionnement quotidien. Il sert pour :
- L'achat et l'activation initiale
- Les mises à jour
- Les vérifications périodiques facultatives
- Le support
- Les sauvegardes cloud optionnelles

---

## 2. Architecture complète

```
INTERNET
  │
  ├── Site MboaCaisse (vente, paiement)
  ├── Mboa License API (licences, activations, entitlements)
  └── Update Server (mises à jour)
       │
       ▼ Activation initiale
       │
       ▼
  ┌────────────────────────┐
  │   RESTAURANT CLIENT    │
  │   PC PRINCIPAL         │
  │                        │
  │   Tauri 2 + Rust       │
  │   Caddy (reverse proxy)│
  │   Axum (REST + WS)     │
  │   SQLx → SQLite        │
  │   Licence locale       │
  │   + Ed25519            │
  └────────────────────────┘
       │
       │ RÉSEAU LOCAL (mboacaisse.local)
       │
  ┌────┼────┐
  │    │    │
  ▼    ▼    ▼
Serv. Cais. Cui.
```

---

## 3. Licence ≠ Serveur

Séparation fondamentale entre deux notions :

| Notion | Rôle |
|--------|------|
| **Licence** | Qui a acheté ? Quel produit ? Quelle édition ? Quels modules ? Quelles versions ? |
| **Activation** | Sur quelle installation ? Quel restaurant ? Quel appareil serveur ? |

Une licence peut avoir plusieurs activations selon les droits du client :

| Édition | Activations |
|---------|-------------|
| Standard | 1 serveur actif |
| Professional | 1 serveur actif |
| Enterprise | Plusieurs serveurs / restaurants |

---

## 4. Les quatre identifiants

### Customer ID
Identifie le client.

```
CUS-8A71F4
```

Exemple : `Restaurant Chez Mboa → CUS-8A71F4`

### Organization ID
Identifie l'entreprise ou le groupe.

```
ORG-2B19D7
```

Permet de gérer :
```
Groupe Mboa
  ├── Restaurant Douala
  ├── Restaurant Yaoundé
  └── Restaurant Bafoussam
```

### License ID
Identifie le contrat commercial.

```
LIC-5C29F8
```

Une organisation peut posséder plusieurs licences.

### Installation ID
Identifie une installation réelle.

```
INS-91E2AC
```

Hiérarchie complète :
```
Organisation
  ├── Licence A
  │     ├── Installation 1
  │     └── Installation 2
  └── Licence B
        └── Installation 3
```

---

## 5. Licence signée cryptographiquement

La licence est un document signé avec **Ed25519**.

```
Mboa License Server (Private Key 🔐)
  │
  ▼ Signature Ed25519
  │
  ▼ Licence signée
  │
  ▼
MboaCaisse (Public Key 🔓)
  │
  ▼ Vérification locale (offline)
```

La clé privée ne doit jamais être incluse dans MboaCaisse.

---

## 6. Exemple de licence

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

---

## 7. Licence perpétuelle et mises à jour

| Élément | Valeur |
|---------|--------|
| Licence | Perpétuelle |
| Mises à jour | Incluses 12 mois |
| Correctifs sécurité | Gratuits (politique support) |
| Après 12 mois | App continue de fonctionner. Pas d'accès aux nouvelles versions sans renouvellement ou upgrade. |

---

## 8. Entitlements (feature gating)

Au lieu de coder l'édition en dur, chaque fonctionnalité est un entitlement :

```rust
#[tauri::command]
fn check_feature(feature: &str, license: &License) -> bool {
    license.features.contains(&feature.to_string())
}
```

Les entitlements sont définis dans la licence côté serveur et vérifiés localement.

---

## 9. Activation initiale

1. L'utilisateur achète MboaCaisse sur le site
2. Un compte est créé
3. La licence est générée
4. L'utilisateur télécharge MboaCaisse
5. À la première installation :
   - Saisie de la clé d'activation
   - Génération d'un `Installation ID` (machine + timestamp)
   - Envoi au License Server (si connecté)
   - Stockage local de la licence signée
6. En mode déconnecté : validation locale différée (grace period)

---

## 10. Vérification en ligne périodique

Périodiquement (configurable, ex. 7 jours), MboaCaisse peut vérifier la validité de la licence en ligne. En cas d'échec de connexion, la licence locale reste valide. En cas de révocation, l'application affiche un avertissement progressif.

---

## 11. Gestion des installations

- À l'activation, un `Installation ID` est généré localement (basé sur la machine)
- Le License Server garde une liste des installations autorisées
- L'utilisateur peut désactiver une installation depuis son dashboard
- En cas de remplacement du PC serveur, une nouvelle installation est créée (l'ancienne peut être désactivée)

---

## 12. Éviter le verrouillage matériel excessif

Le système permet la réactivation sur du nouveau matériel sans intervention du support, dans une limite raisonnable (ex. 3 fois par an). Au-delà, vérification manuelle.

---

## 13. Architecture du License Server

```
License Server
├── API REST (licences, activations, entitlements)
├── Base de données (clients, licences, installations)
├── Dashboard admin
└── Service de vérification
```

Projet indépendant de MboaCaisse (service cloud séparé).

---

## 14. API du License Server

| Endpoint | Méthode | Description |
|----------|---------|-------------|
| `/api/license/activate` | POST | Activer une licence |
| `/api/license/verify` | GET | Vérifier une licence |
| `/api/license/deactivate` | POST | Désactiver une installation |
| `/api/license/list` | GET | Lister les installations actives |
| `/api/update/check` | GET | Vérifier les mises à jour disponibles |

---

## 15. Dashboard administrateur

Le License Server expose un back-office avec :
- Liste des clients et organisations
- Licences actives / expirées / révoquées
- Installations par licence
- Historique des activations
- Génération de licences
- Statistiques

---

## 16. Intégration paiements

Le site commercial s'intègre avec un service de paiement pour :
- Achat de licence
- Renouvellement de mise à jour
- Upgrade d'édition

Le processus :
1. Client visite le site
2. Choisit MboaCaisse et l'édition
3. Paie
4. Compte créé
5. Licence générée
6. Téléchargement disponible

---

## 17. Téléchargements liés à la licence

Les téléchargements sont liés à la licence. Seuls les clients avec une licence active et valide peuvent télécharger la version correspondante. Le License Server gère les entitlements et sert les bons binaires.

---

## 18. Système de mise à jour

Le processus de mise à jour :

1. Arrêt propre d'Axum
2. Sauvegarde de SQLite
3. Téléchargement et installation de la nouvelle version
4. Vérification de l'intégrité (signature)
5. Vérification de la base de données
6. Redémarrage du serveur

Tauri Updater est utilisé comme mécanisme de distribution. Les mises à jour de sécurité restent gratuites même après la période de 12 mois.

---

## 19. Considérations de sécurité

- Signature Ed25519 pour l'intégrité des licences
- Clé privée du License Server uniquement (jamais dans le client)
- Communication API License Server → client via HTTPS
- Vérification de révocation périodique
- Pas de dépendance bloquante à Internet pour le fonctionnement quotidien

---

## 20. Architecture finale recommandée

```
Mboa License Server (cloud)
  │ API REST (HTTPS)
  │
  ▼
MboaCaisse Server (restaurant)
  ├── Licence signée (JSON + Ed25519)
  ├── Vérification locale (offline)
  ├── Feature gating (entitlements)
  └── Mise à jour (Tauri updater)
```

| Avantage | Description |
|----------|-------------|
| Offline-first | Le restaurant fonctionne sans Internet |
| Sécurité | Signature Ed25519, pas de clé privée dans le client |
| Flexibilité | Entitlements granulaires par feature |
| Résilience | Grace period, vérification périodique non bloquante |
| Scalable | Architecture client/serveur pour gérer des groupes |
| Simple | L'utilisateur voit juste une clé d'activation |

# MboaCaisse — AI Memory

## 2026-07-21 Session: Creative Partner — Expansion, Fidélisation, Monétisation

### Concepts clés générés

**Wallet = noyau dur de tout le produit**
- Wallet multi-source (MoMo, cash, gift, cashback, transfer)
- Solde toujours calculé (`SUM(amount)`), jamais stocké
- Clé = numéro téléphone (identifiant fidélité passif)
- Client sans téléphone = ID interne `CLI-XXXX`
- Paiement **avant** validation commande (wallet check)
- Dépôt client optionnel, désactivé par défaut (zone grise régulation)

**Menu public 5 écrans (P0.5)**
1. QR → landing (table ID dans URL)
2. Menu catégories/produits (prix FCFA, pas d'images)
3. Panier (qté, total, bouton commander)
4. Identification téléphone (wallet existant ou création)
5. Confirmation + statut temps réel

**Fidélité sans friction**
- Cashback auto 5% — pas de carte, pas d'app, juste le numéro
- Seuil progressif : 3% → 5% → 8% (game design passif)
- Parrainage : 1000 FCFA sur les deux wallets via "recommandé par"
- QR par table (pas de sélection), pas de step supplémentaire

**6 bundles = feature flags, pas de forks**
- Mboa Cash (encaissement + wallet + fidélité basique)
- Mboa Resto (Cash + MoMo + pré-commande + kitchen + tables)
- Mboa Stock (Resto + inventaire + fournisseurs + multi-dépôt)
- Mboa Traiteur (Resto + planning + acomptes + fiches techniques)
- Mboa Hôtel (Resto + chambres + minibar + facture séjour)
- Mboa Market (Stock + code-barres + inventaire tournant + marge auto)
- Licensing Ed25519 existant (P4) = vérification offline des flags

**Bug du Succès (3 ans, 50 établissements)**
- P0 **wallet_ledger** — table append-only, INSERT-only, backup toutes les 5 min
- P0 **impression queue async** — file d'attente + retry + fallback ticket numérique
- P1 mDNS personnalisable (chezbob.local)
- P2 sync groupe (repoussée — wallet par instance acceptable)

### Décisions d'architecture
- Wallet = un seul par téléphone, multi-sources (pas wallets séparés fusionnés)
- Paiement wallet avant que la commande parte en cuisine (pas de crédit par défaut)
- MoMo = source d'approvisionnement, pas mode de paiement direct
- Impression asynchrone = ne bloque jamais la commande
- wallet_ledger append-only créé avec rétro-compatibilité (rejoue historique)

### Liens
- [FEATURES.md](../../FEATURES.md) — backlog fonctionnel
- [Architecture](../../docs/architecture-mboacaisse.md)
- [Licensing](../../docs/systeme-de-licences.md)

### Règles importantes
- Le téléphone est la clé universelle (pas login, pas carte, pas app)
- Wallet + impression + ledger = triangle de résilience
- Les 6 bundles sont du feature gating, pas du code séparé
- Offline-first : wallet ledger en append-only, backup fréquent

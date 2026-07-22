---
name: MboaCaisse — Public Menu
status: draft
sources:
  - _bmad-output/planning-artifacts/prds/prd-MboaCaisse-2026-07-21/prd.md
  - _bmad-output/planning-artifacts/prfaq-MboaCaisse.md
  - _bmad-output/planning-artifacts/briefs/brief-MboaCaisse-2026-07-21/brief.md
updated: 2026-07-21
---

# MboaCaisse — Experience Spine (Public Menu)

> Mobile web (4" phone) pour le client QR. Desktop pour caisse/cuisine. Nuxt 4 + Nuxt UI v4. DESIGN.md est la référence visuelle — ce document décrit le comportement.

## Foundation

**Client QR :** navigateur mobile (Chrome, Safari), écran 4", tactile uniquement, pas d'app installée, pas de session persistante. Le client arrive par scan QR (`http://{host}:{PORT}/menu?table={id}`). Pas de WebSocket en V1 — le flux est synchrone, request-response HTTP. Le statut commande est un confort visuel (rafraîchissement manuel).

**Caisse/Cuisine :** desktop navigateur plein écran, clavier + souris. Kitchen display reçoit les commandes par polling HTTP.

## Information Architecture

6 écrans, flux linéaire avec possibilité de retour arrière :

```
[QR scan] → Landing → Menu (catégories + produits) → Panier → Identification + Paiement → Confirmation
                                                                                              ↓
                                                                                         [Statut]
```

| Surface | Atteinte depuis | But |
|---|---|---|
| Landing QR | Scan QR / `GET /menu?table={id}` | Affiche numéro de table ou mode takeaway, CTA "Voir le menu" |
| Menu | Landing → bouton / navigation directe | Parcourir catégories et produits, prix FCFA, options de personnalisation |
| Panier | Menu → ajout produit | Réviser la sélection, modifier quantités, entrer téléphone |
| Identification | Panier → bouton "Commander" | Saisie téléphone (ou déjà connu), choix du mode de paiement |
| Confirmation | Paiement validé | Numéro commande, montant, remerciement |
| Statut | Confirmation / URL directe | "En préparation" / "Prête" — pas de polling, rafraîchissement manuel |

## Voice and Tone

Microcopy publique. Le ton est chaleureux mais efficace — pas de blagues, pas d'excès de politesse.

| Situation | Microcopy |
|---|---|
| Landing | "Bienvenue chez [Établissement]" — ou "Bon appétit !" si commande déjà en cours |
| Menu vide (catégorie) | "Rien ici pour l'instant" |
| Panier vide | "Votre panier est vide. Ajoutez des articles depuis le menu." |
| Téléphone requis | "Entrez votre numéro pour commander" |
| Paiement wallet insuffisant | "Solde insuffisant ({solde} FCFA). Paiement au comptoir." |
| Commande validée | "Commande #{N} confirmée. Merci {prénom} !" |
| Erreur réseau | Pas de message — la page ne charge pas. Le commerçant a un panneau physique. |

## Component Patterns

Comportement. Le visuel vit dans DESIGN.md.

| Composant | Usage | Règles comportementales |
|---|---|---|
| Barre catégorie | Menu | Tap pour filtrer les produits. Active en surbrillance. |
| Fiche produit | Menu (liste) | Nom + prix FCFA. Tap → ajoute au panier ou ouvre les options (si personnalisation). |
| Collapsable options | Fiche produit (si perso) | Cuisson, sauce, quantité. Caché par défaut. Tap pour déplier. |
| Panier badge | Header (Menu, Panier) | Nombre d'articles, visible même si 0. |
| Ligne panier | Panier | Nom, quantité (- / +), prix ligne, bouton supprimer. |
| Champ téléphone | Identification | Input numérique, validation format téléphone (9 chiffres, préfixe auto ?). |
| Choix paiement | Identification | Deux boutons : "Payer avec mon wallet" / "Payer au comptoir". Wallet désactivé si solde insuffisant + explication. |
| Bouton commander | Panier / Identification | Désactivé tant que téléphone pas saisi. |
| Statut visuel | Confirmation / Statut | Bande de progression 3 états : commandée → en préparation → prête. Mis à jour au chargement. |

## State Patterns

| État | Surface | Traitement |
|---|---|---|
| Première visite (pas de cookie) | Landing | Affiche le CTA "Voir le menu". Pas de téléphone connu. |
| Retour (cookie présent, même session) | Menu | Affiche "Bonjour {prénom} !" subtil en haut. Téléphone pré-rempli au passage en caisse. |
| Catégorie vide | Menu | "Rien ici pour l'instant" — pas de produits fantômes. |
| Panier vide | Panier | Texte + bouton "Retour au menu". |
| Solde wallet insuffisant | Paiement | Wallet grisé + "Solde : {solde} FCFA. Paiement au comptoir recommandé." |
| Nouveau client (téléphone inconnu) | Identification | Crée compte avec 0 FCFA. Propose paiement au comptoir. |
| Client connu | Identification | Affiche le solde + cashback accumulé. Wallet disponible. |
| Commande en cours | Confirmation / Statut | "Votre commande est en cuisine" / "Prête ! Bon appétit !" |
| Table non trouvée | Landing (URL invalide) | "Table non trouvée. Scannez le QR de votre table ou demandez au serveur." |
| Erreur serveur | Toute page | Écran minimaliste : "Une erreur est survenue. Réessayez ou parlez au serveur." Pas de stack trace. |

## Interaction Primitives

**Client QR — tactile uniquement.**

- Tap = action primaire (ajouter au panier, valider, naviguer).
- Pas de hover (mobile), pas de drag, pas de swipe.
- Retour arrière : bouton "←" dans le header ou geste natif du navigateur.
- Sortie : le client ferme l'onglet. Pas de session persistante au-delà de la navigation.

**Cuisine/Caisse — clavier + souris.**

- Click = action. Tab navigation entre les champs.
- Raccourcis clavier à définir dans le spine caisse (hors scope menu public).

**Banni partout :** drag & drop, swipe, hover-only comme seul moyen d'accéder à une action, session qui expire sans prévenir, modale empilée.

## Accessibility Floor

- WCAG 2.2 AA sur les surfaces client (mobile) et caisse (desktop).
- Zones tactiles ≥ 44×44px (critique sur écran 4").
- Contraste des textes sur fond : le DESIGN.md hérite des valeurs par défaut de Nuxt UI v4.
- `Tab` order logique dans les formulaires (téléphone, choix paiement).
- `aria-live` sur les changements de statut commande.
- Pas de texte en image. Pas d'icône sans fallback texte.

## Key Flows

### Flow 1 — Jean commande depuis sa table (UJ-3, climax : la première bière qui arrive sans attendre le serveur)

1. Jean s'assoit à la table 4, voit le QR code plastifié au milieu de la table.
2. Il sort son téléphone, se connecte au WiFi "MboaCaisse", scanne le QR.
3. Landing page : "Bienvenue chez [Établissement] — Table 4". Bouton "Voir le menu".
4. Menu : 3 catégories (Bières, Plats, Sucreries). Il tape "Bières". Prix en FCFA bien visibles.
5. Il ajoute 2 bières (pas d'options). Puis tape "Plats", ajoute 1 planteur (cuisson : "bien cuit" via collapsable).
6. Il tape l'icône panier en haut. Récap : 2 bières + 1 planteur → 3 500 FCFA.
7. **Climax :** Jean entre son téléphone (nouveau client). L'écran lui dit "Bienvenue ! Solde : 0 FCFA. Paiement au comptoir sélectionné." Il tape "Confirmer la commande". L'écran passe à "Commande #42 confirmée ! Merci Jean ! Votre commande est en cuisine." Dans le même temps, la cuisine voit "TABLE 4 — 2 Bières, 1 Planteur (bien cuit)" sur l'écran kitchen display. Jean n'a pas appelé le serveur une seule fois.
8. Le serveur voit la commande passer à "prête" sur son écran. Il va chercher les bières, traverse la salle, et appelle : "Jean, table 4 !". Jean lève la main. La bière arrive — sans notification, sans polling, juste un humain qui fait son travail.
8. **Échec :** solde insuffisant si wallet → "Solde : 2 000 FCFA, total : 3 500 FCFA. Paiement au comptoir recommandé." Jean tape "Payer au comptoir", va chercher sa commande au bar.

### Flow 2 — Aicha prend à emporter (takeaway QR, climax : gagner 10 minutes sur sa pause déj)

1. Aicha entre dans l'établissement, voit le QR "À emporter" au comptoir.
2. Scanne → Landing avec "Table 4" remplacé par "À emporter".
3. Même flux que Jean : menu → ajout → téléphone → confirmation.
4. **Climax :** pas de file d'attente au comptoir. Sa commande est déjà passée quand elle arrive pour payer. Elle dit son numéro, le caissier valide, la cuisine prépare. Elle gagne 10 minutes.

## Responsive & Platform

Le client QR est **mobile uniquement** (portrait, 320–480px). Pas de version desktop du menu public — le UX desktop est le dashboard caisse/admin.

| Breakpoint | Comportement |
|---|---|
| < 480px (mobile portrait) | Taille d'écran cible. Pleine largeur. |
| ≥ 480px (mobile paysage / phablet) | Menu toujours centré, marges augmentées. Maximum 600px de largeur de contenu. |
| Desktop (caisse/cuisine) | Responsive dédiée (dashboard, hors scope menu public). |

### Anti-patterns

- **Rejeté :** carrousel d'images. Pas d'images (offline, poids). Catégories en liste verticale.
- **Rejeté :** swipe pour naviguer entre écrans. Tap uniquement.
- **Rejeté :** notification push, son, vibration, WebSocket. L'établissement est petit — le serveur voit l'écran et appelle.
- **Rejeté :** QR dynamique qui change. QR imprimé = URL fixe.
- **Rejeté :** compte client, email, mot de passe, app. Le téléphone est le seul identifiant.

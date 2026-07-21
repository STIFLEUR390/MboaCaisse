---
name: MboaCaisse
description: "POS et gestion pour établissements Africains — wallet Mobile Money, zéro abonnement, LAN-first"
colors:
  primary-green: '#16A34A'
  primary-green-hover: '#15803D'
  surface-base: '#FAFAF9'
  surface-card: '#FFFFFF'
  surface-overlay: '#0000001A'
  text-primary: '#1C1917'
  text-secondary: '#57534E'
  text-on-primary: '#FFFFFF'
  text-warning: '#DC2626'
  border-light: '#E7E5E4'
  success: '#16A34A'
  warning: '#F59E0B'
typography:
  display:
    fontFamily: Inter
    fontSize: 28px
    fontWeight: '700'
    lineHeight: '1.2'
  heading:
    fontFamily: Inter
    fontSize: 18px
    fontWeight: '600'
    lineHeight: '1.3'
  body:
    fontFamily: Inter
    fontSize: 14px
    fontWeight: '400'
    lineHeight: '1.5'
  caption:
    fontFamily: Inter
    fontSize: 12px
    fontWeight: '400'
    lineHeight: '1.4'
  price:
    fontFamily: Inter
    fontSize: 16px
    fontWeight: '700'
    lineHeight: '1.3'
rounded:
  sm: 4px
  md: 8px
  lg: 12px
  full: 9999px
spacing:
  '1': 4px
  '2': 8px
  '3': 12px
  '4': 16px
  '5': 20px
  '6': 24px
  '8': 32px
  '10': 40px
  gutter-mobile: 16px
components:
  button-primary:
    backgroundColor: '{colors.primary-green}'
    color: '{colors.text-on-primary}'
    borderRadius: '{rounded.md}'
    padding: '{spacing.3} {spacing.6}'
    fontSize: '{typography.body.fontSize}'
    fontWeight: '600'
  button-secondary:
    backgroundColor: transparent
    border: '1px solid {colors.border-light}'
    color: '{colors.text-primary}'
    borderRadius: '{rounded.md}'
    padding: '{spacing.3} {spacing.6}'
  card-product:
    backgroundColor: '{colors.surface-card}'
    borderRadius: '{rounded.md}'
    padding: '{spacing.3} {spacing.4}'
    border: '1px solid {colors.border-light}'
  input-phone:
    backgroundColor: '{colors.surface-card}'
    border: '1px solid {colors.border-light}'
    borderRadius: '{rounded.md}'
    padding: '{spacing.3} {spacing.4}'
    fontSize: '{typography.body.fontSize}'
  progress-step:
    activeColor: '{colors.primary-green}'
    inactiveColor: '{colors.border-light}'
    labelColor: '{colors.text-secondary}'
    labelFontSize: '{typography.caption.fontSize}'
---

# MboaCaisse — Design Spine (Public Menu)

> [ASSUMPTION : Nuxt UI v4 fournit les valeurs par défaut (typo, espacement, ombres). Ce document définit les *deltas* — la couche de marque MboaCaisse. Les sections omises héritent de Nuxt UI v4.]

## Brand & Style

[ASSUMPTION : la marque MboaCaisse est ancrée dans la confiance et la simplicité. Pas de fioritures. Le design doit rassurer un commerçant qui a perdu de l'argent avec des solutions compliquées — et donner envie au client de commander. Le vert comme couleur primaire évoque la croissance et l'argent. Le zinc comme neutre, chaud et terreux — pas un blanc clinique.]

MboaCaisse est utilitaire chaleureux. Le client QR doit charger en < 1s, se lire sur un écran 4" en plein soleil, et ne jamais faire douter du prix. Le dashboard caisse doit permettre un encaissement en 3 taps. Rien n'est décoratif — tout sert une transaction plus rapide, plus fiable, plus agréable.

**Client QR :** minimal, lisible, pas d'images. Le prix est l'élément le plus important du menu. Le bouton "Commander" est la seule chose colorée.

**Caisse/Cuisine :** haute densité d'information, actions visibles sans scroll, pas de surprises.

## Colors

Palette inspirée de la croissance et de la confiance — le vert pour l'argent, le zinc pour le quotidien.

| Token | Usage | Ne PAS utiliser pour |
|---|---|---|
| `primary-green` | Boutons CTA, soldes wallet, highlight statut "prête" | Texte long, arrière-plan (sauf bouton) |
| `surface-base` | Fond de page (client QR) | — |
| `surface-card` | Cartes produit, panier, modale | — |
| `text-primary` | Titres, prix, corps | — |
| `text-secondary` | Descriptions, labels, timestamps | Prix, CTA |
| `border-light` | Séparateurs, bordures de cartes | — |
| `text-warning` | Solde insuffisant, erreur | — |
| `success` | Confirmation, statut "prête" | — |
| `warning` | Stock bas, seuil (dashboard) | — |

[ASSUMPTION : les couleurs reprennent la config Nuxt UI v4 du projet (green primary, zinc neutral). Les tokens ci-dessus en sont un sous-ensemble nommé.]

## Typography

Inter — lisible à 14px sur écran 4", bonne graisse africaine.

|[ASSUMPTION]| Rôle | Mobile | Usage |
|---|---|---|---|
| `display` | 28px / 700 | Titre landing, confirmation | Un seul par page |
| `heading` | 18px / 600 | Noms de catégories, titres section | — |
| `body` | 14px / 400 | Descriptions, labels | Taille par défaut |
| `price` | 16px / 700 | Prix FCFA | Toujours en vert primaire |
| `caption` | 12px / 400 | Numéro commande, timestamps | Jamais pour des actions |

Règle : le prix est toujours en `price` (visible, gras, vert). Jamais de prix sans le suffixe "FCFA".

## Layout & Spacing

**Client QR (mobile) :** 16px de marge latérale (`gutter-mobile`). Contenu centré, max-width 600px. Espacement vertical 16px entre les éléments, 24px entre les sections.

**Composant clé :** le bouton "Commander" est toujours en bas de l'écran (sticky footer sur les écrans de saisie — panier, identification). Pas de risque de le manquer.

**Menu :** catégories en barre horizontale scrollable. Produits en liste verticale (pas de grille — une colonne, lisible).

## Elevation & Depth

Pas de shadow lourdes. Nuxt UI v4 gère les ombres par défaut. Sur mobile client, les cartes produit ont un `box-shadow` très léger — juste assez pour se détacher du fond. Pas de superposition de modales.

## Shapes

| Token | Usage |
|---|---|
| `sm` (4px) | Input téléphone, badges |
| `md` (8px) | Cartes produit, boutons, modales |
| `lg` (12px) | Header, conteneurs |
| `full` | Avatars (aucun en V1 client), badges statut |

## Components

### Button — Primary

Fond vert (`primary-green`), texte blanc, 8px de rayon, padding 12px 24px. Le seul bouton coloré de l'écran. État désactivé : opacité 50%.

### Button — Secondary

Pas de fond, bordure grise (`border-light`), texte primaire. Pour les actions secondaires ("Retour au menu", "Payer au comptoir").

### Card — Product

Fond blanc, bordure grise fine, 8px rayon. Padding 12px 16px. Nom du produit en `body`, prix en `price` (noir `text-primary`, aligné à droite). Pas d'image. Tap → feedback visuel (scale(0.97) 150ms).

### Collapsable Options

Icône chevron à droite. Tap → déplie le contenu avec animation simple. Padding uniforme.

### Progress — Statut commande

3 steps horizontaux : commandée ● — en préparation ● — prête ●. Step actif en `primary-green`. Step complété en `success`. Step futur en gris clair. Label sous chaque step.

### Input — Phone

Fond blanc, bordure grise, 8px rayon, padding 12px 16px. Texte en `body`. Focus → bordure `primary-green`. Clavier numérique natif (`type="tel"` ou `inputmode="numeric"`).

## Do's and Don'ts

| ✅ Do | ❌ Don't |
|---|---|
| Prix en `price` (16px/700) avec FCFA | Prix en gris, en petit, ou sans FCFA |
| Bouton CTA toujours en bas (sticky) | CTA perdu en milieu de page |
| Une colonne en mobile | Grille 2 colonnes sur 4" |
| Catégories en barre horizontale | Catégories en dropdown |
| Retour arrière visible dans le header | Piéger l'utilisateur sans bouton retour |
| Message "Solde insuffisant" clair | Message vague ou technique |
| Confirmation avec numéro commande visible | Confirmation sans référence |

---
title: "PRFAQ: MboaCaisse"
status: "press-release"
created: "2026-07-21"
updated: "2026-07-21"
stage: 2
inputs:
  - "docs/architecture-mboacaisse.md"
  - "docs/systeme-de-licences.md"
  - "_bmad-output/planning-artifacts/briefs/brief-MboaCaisse-2026-07-21/brief.md"
---

# MboaCaisse : votre caisse sur votre PC, pas dans le cloud. Wallet Mobile Money, zéro abonnement, zéro Internet.

**Un serveur de caisse qui tient sur n'importe quel PC. Accessible depuis tout le réseau local — téléphone, tablette, écran cuisine. Client wallet Orange Money et MTN MoMo intégré. Impression thermique, fidélité sans carte, licence à vie. Tout tourne sans Internet.**

**Alpha fermé — premiers tests en cours à Douala.**

Les petits commerces de quartier en Afrique francophone tiennent leur caisse au cash et au cahier. Le propriétaire ferme le soir sans savoir combien il a vendu. Le caissier peut voler sans laisser de trace. Un client veut payer en Mobile Money ? Le commerçant note le numéro de transaction sur un bout de papier. Les solutions cloud coupent au premier outage Internet. Les terminaux importés coûtent 500 000 FCFA et ne savent ni gérer un stock ni fidéliser un client. Résultat : le propriétaire pilote son affaire à l'aveugle.

MboaCaisse change ça. Vous avez un PC — même un vieux qui traîne — et un routeur WiFi ? Branchez une imprimante thermique USB, installez MboaCaisse en 5 minutes, et tout l'établissement est connecté. Le caissier encaisse sur l'écran principal. La cuisine reçoit les commandes sur une tablette. Les serveurs prennent les commandes depuis leur téléphone. Le propriétaire consulte les rapports depuis chez lui. Les clients paient en cash, Orange Money ou MTN MoMo — le wallet déduit automatiquement le solde. Les tickets s'impriment en un clic. Pas d'abonnement, pas de cloud, pas de frais mensuels. Tout fonctionne sans Internet.

> "Le POS n'est pas un luxe pour supermarché. C'est un outil de base pour tout commerce, même le plus petit. Un commerçant à Douala ou Abidjan doit pouvoir encaisser, suivre son stock et savoir exactement combien il gagne — sans attendre que l'Internet revienne, sans payer 500 000 FCFA un terminal."
> — Herold, Fondateur

### Comment ça marche

1. **Jour 1 — Installation.** Téléchargez MboaCaisse sur votre PC Windows ou Linux. Lancez l'installeur. L'icône apparaît dans la barre système — la caisse est prête. Pas besoin de dédier le PC : il peut servir aux devoirs des enfants pendant que MboaCaisse tourne en arrière-plan.
2. **Ajoutez vos produits.** Créez vos catégories (boissons, plats), définissez les prix. Tout est sur votre PC, pas chez un hébergeur.
3. **Encaissement.** Le caissier sélectionne les articles, choisit le paiement (espèces, Orange Money, MTN MoMo, wallet client). Le ticket s'imprime en thermique. La commande part à la cuisine.
4. **Wallet client.** Le client donne son numéro de téléphone à l'inscription. Il dépose de l'argent en espèces ou via Mobile Money. Chaque mouvement est écrit dans un journal immuable — pas de solde qui peut être trafiqué.
5. **Fidélité automatique.** Cashback 3 % → 5 % → 8 % selon les paliers de dépenses. Parrainage : 1 000 FCFA reversés aux deux parties. Pas de carte, pas d'appli.
6. **Fin de journée.** Le propriétaire consulte le rapport de ventes, les totaux par caissier, les ruptures de stock sur son téléphone depuis chez lui.

> "Avant MboaCaisse, je fermais la caisse sans savoir combien j'avais fait. Mes caissières notaient sur un carnet. Je perdais de l'argent chaque jour sans pouvoir le prouver. Maintenant, je vois tout sur mon téléphone depuis la maison. Le cashback a doublé mes clients réguliers en deux mois."
> — Patrick N., Propriétaire de bar, Yaoundé

### Premiers pas

MboaCaisse est en alpha fermé — premiers tests en cours à Douala. Vous voulez essayer ? Pas de paiement, pas de carte — 30 jours gratuits, installation assistée par WhatsApp, formation incluse. Si ça ne vous fait pas gagner plus d'argent, on désinstalle et vous retournez au carnet. Après l'essai, la licence perpétuelle commence à 120 000 FCFA pour l'édition Cash (mises à jour incluses 12 mois). C'est ce que vous perdez en un mois de vols caissier non tracés.

**Mode dégradé :** si le PC plante ou que l'électricité coupe, la base de données est sauvegardée automatiquement chaque jour. Vous installez MboaCaisse sur un autre PC, vous restaurez le backup en un clic (guidé par WhatsApp si besoin), et vous repartez. Pas de perte.

**Le PC n'est pas dédié ?** Pas grave. MboaCaisse tourne en service Windows/Linux en arrière-plan. Le propriétaire peut laisser ses enfants faire leurs devoirs pendant que la caisse continue de fonctionner. Si le service s'arrête, une notification part sur son téléphone.

---

## Customer FAQ

### Q : Est-ce que ça marche vraiment sans Internet ?

A : Oui, c'est le cœur du produit. Le serveur Axum tourne sur votre PC, la base SQLite est locale, tous les appels API restent sur le réseau local. Vous pouvez encaisser, imprimer, gérer le stock et consulter les rapports même si l'ISP est en panne pendant une semaine. Internet n'est nécessaire que pour l'activation initiale de la licence et les mises à jour facultatives.

### Q : Mon téléphone et ma tablette peuvent-ils accéder à la caisse ?

A : Oui. Tout appareil connecté au WiFi de l'établissement peut accéder à MboaCaisse via le navigateur — Chrome, Safari, Firefox, Edge. Votre caissier utilise l'écran principal. Les serveurs prennent les commandes depuis leur téléphone. La cuisine reçoit les tickets sur une tablette dédiée. Pas d'appli à installer, pas de configuration IP : le serveur se découvre automatiquement via `mboacaisse.local`.

### Q : Quels modes de paiement sont acceptés ?

A : Espèces, Orange Money, MTN MoMo, et wallet client MboaCaisse. Le wallet peut être alimenté par Mobile Money ou espèces. Chaque mouvement est écrit dans un journal qui enregistre tout — impossible de perdre ou trafiquer un centime. Vous pouvez tout vérifier à tout moment. Les paiements sont automatiquement déduits du wallet avant que la commande parte en cuisine.

### Q : Combien ça coûte vraiment ?

A : Licence perpétuelle, pas d'abonnement. Édition Cash (POS + wallet + rapports) : 120 000 FCFA, mises à jour incluses 12 mois. Édition Resto (ajoute écran cuisine, commandes par table, planning serveur) : 180 000 FCFA. Édition Stock (ajoute inventaire, commandes fournisseurs, alerte seuil) : 240 000 FCFA. Après 12 mois, le logiciel continue de fonctionner ; seules les nouvelles versions nécessitent un renouvellement. Les correctifs de sécurité sont gratuits.

### Q : Et si mon PC tombe en panne ?

A : La base SQLite peut être sauvegardée automatiquement (quotidien + avant mise à jour) sur un disque USB ou un dossier partagé. Installez MboaCaisse sur un nouveau PC, restaurez le backup, et vous êtes opérationnel en 15 minutes. Le système de licence permet la réactivation sur nouveau matériel (3 fois par an sans intervention support).

### Q : Mes clients peuvent-ils voir le menu et commander depuis leur téléphone ?

A : Oui. Chaque table peut afficher un QR code. Le client scanne, voit le menu public, consulte le solde de son wallet, et peut passer commande. Pas d'appli à télécharger, pas de compte client à créer, pas de bluetooth à appairer.

### Q : Y a-t-il un support en cas de problème ?

A : Assistance par WhatsApp et email. Les diagnostics réseau sont intégrés (vérification WiFi, mDNS, base de données, WebSocket). Le journal d'activité complet facilite le débogage à distance.

---

## Internal FAQ

### Q : Quelle est la différence réelle avec HandLit POS et Velko POS qui font déjà de l'offline + Mobile Money ?

A : Deux différences fondamentales. (1) **Architecture.** HandLit est une app Android (APK sur téléphone). Velko est web-first avec sync. MboaCaisse est un serveur Tauri/Rust natif sur PC qui sert toute l'équipe via le réseau local — pas de sync, pas de dépendance téléphone, pas de batterie qui se vide. (2) **Multi-écran natif pendant le service.** L'écran caisse, l'écran cuisine, la tablette serveur, le menu client via QR — tout est simultané et temps réel sur le réseau local, sans latence de synchronisation cloud. Le client wallet avec ledger append-only et le feature gating par licence signée Ed25519 sont des barrières techniques que les concurrents Android/web n'ont pas. HandLit et Velko ont validé le marché — MboaCaisse apporte une maturité technique de serveur professionnel là où ils apportent une solution mobile.

### Q : Le marché est-il déjà trop encombré ?

A : Non. Le segment est naissant mais fragmenté. HandLit (500 stores), Velko (5 200 commerçants), Djouri POS, TigiPOS, digabloPos — personne n'a une part dominante. Le marché total adressable (bars, restos, épiceries en Afrique francophone) se compte en centaines de milliers d'établissements. La majorité utilise encore le cash et le cahier. Le problème n'est pas la concurrence mais l'adoption : convaincre un commerçant de passer du carnet à un POS. La barrière n'est pas technique mais commerciale et éducative.

### Q : Pourquoi Rust/Tauri plutôt qu'une PWA ou une app Android comme les concurrents ?

A : Le PC de l'établissement est le point d'ancrage : il est branché au secteur, ne craint pas la batterie, a un port USB pour l'imprimante thermique, et peut servir 10+ écrans simultanément sans ralentir. Rust nous donne des performances natives, une consommation mémoire faible (critique sur un PC de 5 ans), une génération ESC/POS directe sans passer par le navigateur, et une empreinte binaire de quelques Mo. Tauri nous donne la fenêtre native, les plugins système (impression, notifications, store, autostart), et la possibilité de servir le frontend Nuxt via l'HTTP embarqué. Une PWA ne peut pas imprimer en thermique natif ni accéder aux ports USB.

### Q : Le wallet client avec ledger append-only est-il un surcoût technique justifié ?

A : Oui, c'est une décision de conception critique. Un wallet stocké comme solde mutable (`balance = 50 000`) est vulnérable à la corruption silencieuse, aux bugs de concurrence, et à l'absence d'audit. Le ledger append-only (`INSERT INTO wallet_ledger (client_id, amount, type, reference) VALUES (...)`) signifie que le solde est toujours `SUM(amount)` d'une séquence de transactions immuables. C'est plus cher en stockage (quelques Mo au pire pour des milliers de transactions) mais ça garantit l'intégrité comptable — non-négociable pour un produit qui gère de l'argent réel sans connexion cloud pour sauvegarder les données.

### Q : Comment gérer la distribution et le support sans présence physique en Afrique ?

A : La distribution est le vrai problème. HandLit et Velko distribuent par APK Android sur WhatsApp — un tap, zéro friction. MboaCaisse demande un PC, un téléchargement, une installation. La friction est 10× plus haute. La solution n'est pas de copier leur canal mais d'en trouver un où notre avantage technique compte.

**Canal alpha : les grossistes boissons.** Brasseries du Cameroun, Guinness, distributeurs de quartier — ils livrent les bars chaque semaine, ont une liste de clients et une relation de confiance. Si un grossiste recommande MboaCaisse à 3 de ses meilleurs clients, c'est 10× plus efficace qu'un commerçant tombant sur le site. Le grossiste a un intérêt objectif à ce que ses commerçants aient une trésorerie saine (et qu'ils continuent à commander).

**Hypothèse produit : module grossiste gratuit.** Un tableau de bord qui montre au grossiste les volumes commandés par ses clients (pas les données de vente, juste les tendances de réapprovisionnement). Ça lui donne un levier de fidélisation — "je t'aide à mieux vendre" → "tu m'achètes plus". Si le module est utile, le grossiste devient canal de distribution naturel.

**Plan B : réseau personnel.** Un cousin qui tient un bar à Mvog-Mbi, un ancien collègue qui a ouvert un snack à Bastos. Pas scalable mais donne le premier cas d'usage réel, les premières corrections bugs, le premier témoignage authentique.

**Le vrai défi : le passage de 1 à 10.** Un alpha content en parle à son grossiste → le grossiste en parle à 3 autres bars → le bouche-à-oreille B2B commence. Ça suppose que le grossiste joue le jeu. Le module grossiste est conçu pour ça.

### Q : Le PC familial n'est pas dédié à la caisse. Comment gérer ?

A : C'est une objection réelle — dans 80 % des petits commerces, le PC sert à tout le monde, y compris aux devoirs des enfants. MboaCaisse est conçu pour ça : le serveur Axum tourne comme un service Windows/Linux en arrière-plan, sans bloquer l'écran. L'icône dans la barre système signale que la caisse est active. Les enfants font leurs devoirs. Le propriétaire surveille les ventes depuis son téléphone. Si le service s'arrête (PC éteint, plantage), une notification part sur son téléphone. Le mode headless (pas de fenêtre, juste le service) sera disponible en V1.

### Q : Le module grossiste est-il une bonne idée ou une distraction ?

A : C'est une idée qui mérite d'être testée en alpha, pas implémentée en V1. Si un alpha grossiste montre de l'intérêt, on construit. Sinon, on ne force pas. Le risque est de construire un produit B2B alors que le marché principal est B2C (le commerçant). La priorité numéro un est que MboaCaisse marche parfaitement pour le bar qui l'utilise. Si c'est le cas, le canal grossiste peut venir après.

### Q : La licence perpétuelle à 120 000 FCFA est-elle tenable économiquement ?

A : Oui, si le volume suit. À 100 clients la première année, c'est 12 M FCFA (~18 000 €) de revenus — suffisant pour couvrir les coûts d'infrastructure (License Server, domaine, support) et dégager une marge. L'économie est délibérée : un prix trop bas dévalorise le produit, un prix trop haut exclut le marché cible. 120 000 FCFA, c'est le prix d'un smartphone d'entrée de gamme, pas d'un terminal POS importé. Les revenus récurrents viendront des renouvellements de mise à jour (taux de rétention à confirmer en alpha) et des upgrades d'édition.

---

## The Verdict

**Ce qui est forgé :**
- Vision produit cohérente et ancrée dans un vrai problème vécu
- Architecture technique qui justifie chaque choix (Rust/Tauri/LAN-first/ledger)
- Différenciation claire face aux concurrents Android/web
- Modèle de licence perpétuelle adapté au pouvoir d'achat local
- Feature gating qui permet un seul binaire pour tous les segments

**Ce qui demande plus de chaleur :**
- **Distribution : le vrai mur.** HandLit/Velko distribuent par APK WhatsApp (0 friction). MboaCaisse demande téléchargement + installation PC (10× friction). La stratégie grossiste boissons est la piste la plus prometteuse — elle utilise un réseau existant avec relation de confiance. Mais elle n'est pas testée. Risque : arriver en V1 sans canal de distribution fonctionnel.
- **PC non dédié.** 80 % des petits commerces partagent le PC. La solution technique (service headless, notification si arrêt) existe. La question est : est-ce que le propriétaire accepte de laisser un logiciel de caisse tourner sur le PC familial ? Ça se joue sur la confiance et l'habitude. Le mode headless + backup automatique réduisent le risque perçu.
- **1 → 10 : le vrai gap.** Un alpha content via réseau perso (cousin, ami) est facile. Le passage à 10 clients via grossiste est le premier vrai test de la thèse de distribution. Si ça marche, le bouche-à-oreille B2B peut scaler. Si ça échoue, MboaCaisse reste un outil génial utilisé par 3 copains.
- **Éducation au passage au numérique.** Le concurrent n'est pas HandLit ou Velko — c'est le carnet et le cash. Le commerçant doit voir une raison de payer 120 000 FCFA alors que le carnet est gratuit. La promesse "30 jours gratuits, installation assistée, désinstallation si pas de gain prouvé" est la seule chose qui fait bouger un commerçant qui a déjà tout essayé.

**Fissures potentielles :**
- **Économique : le coût d'acquisition non résolu.** 100 clients/an, 12M FCFA est un calcul de revenu, pas de rentabilité. Chaque alpha nécessite installation assistée WhatsApp + formation (2h minimum, temps du fondateur). À 100 clients, c'est 200h de support direct sans compter le debugging. Ce temps ne scale pas sans embauche ou passation à un réseau de revendeurs formés. Le modèle économique n'est pas faux — il est incomplet tant que le coût d'acquisition réel n'est pas mesuré sur les 10 premiers clients.
- HandLit (500 stores) et Velko (5 200 commerçants) ont un an d'avance, des retours terrain quotidiens, des revendeurs dans les quartiers. MboaCaisse arrive avec une meilleure architecture mais zéro relation terrain. Sous-estimer cette avance est dangereux.
- Le canal grossiste est une hypothèse forte. Les grossistes boissons sont-ils prêts à recommander un logiciel de caisse à leurs clients ? Quel intérêt concret pour eux (au-delà du module gratuit) ? À valider avant d'investir.
- Le marché s'accélère. HandLit, Velko, Djouri, digablo — tous grandissent. La fenêtre pour construire un avantage concurrentiel durable se referme.

---

<!-- coaching-notes-stage-2 -->
- **Ton corrigé** : passage du registre "Western PR" vers voix de commerçant. Titre + sous-titre retravaillés pour parler cash, pas concepts. "Manager" remplacé par "propriétaire" partout.
- **Alpha positionné** : pas un "lancement" mais un alpha fermé à Douala. Nombre non quantifié dans le texte final — "premiers tests en cours" seulement. La quantification viendra quand elle sera vraie.
- **Stratégie distribution** : canal grossiste boissons identifié comme levier principal. Module grossiste gratuit proposé comme hypothèse produit (pas engagé en V1). Passage 1→10 identifié comme le vrai gap.
- **PC familial** : mode headless + notification arrêt + backup automatique comme réponse à l'objection du PC partagé. Le service ne bloque pas l'écran.
- **Prix** : comparaison changée de "prix d'un smartphone" → "ce que vous perdez en vols caissier non tracés".
- **30 jours gratuits** : reformulé pour éliminer toute suspicion de carte bancaire cachée. Installation assistée par WhatsApp incluse.
- **Concurrents** : HandLit (500 stores, APK WhatsApp) et Velko (5 200 commerçants, 14 pays) identifiés comme menaces sérieuses avec avance terrain réelle. MboaCaisse a une meilleure architecture mais zéro relation terrain — c'est le vrai déséquilibre.
- **Décisions rejetées** : distribution purement numérique (trop inefficace), comparaison smartphone (hors sujet), ton TechCrunch (pas crédible).

# Blind Hunter — Adversarial Review Findings

1. **Route prefix collision dans required_permission** : `/api/users` match aussi `/api/users-export` ou `/api/users-settings` via `starts_with`. Si une route future suit ce pattern, elle héritera de Permission::ManageUsers au lieu du fallback Permission::All. Utiliser `path.starts_with("/api/users/") || path == "/api/users"` pour être précis.

2. **Permission unique pour GET et POST sur la même route** : La fonction `required_permission` ne distingue pas les verbes HTTP. `GET /api/settings` (lecture) et `PATCH /api/settings` (écriture) requièrent tous deux ManageSettings. Si on veut séparer read/write plus tard, il faudra repenser le mapping.

3. **Aucune protection contre les modifications concurrentes** : `update_user` et `delete_user` n'ont pas d'optimistic locking ou de version. Deux admins modifiant le même utilisateur simultanément peuvent écraser les changements de l'autre. Acceptable pour LAN alpha, mais à documenter.

4. **DELETE expose le dernier admin protégé mais pas la race condition** : Si deux admins suppriment des comptes admin simultanément, chacun voit `admin_count > 1` mais le dernier pourrait être supprimé par les deux opérations. Ajouter une transaction atomique pour la vérification + suppression.

5. **Le rôle admin est vérifié côté frontend mais pas côté API pour les pages** : `minRole` dans `definePageMeta` filtre la navigation, mais n'empêche pas un utilisateur de taper l'URL directement. Le middleware `admin.ts` protège ça ✅, mais `settings.vue` n'a que `minRole: "admin"` sans middleware admin — la route `/settings` reste accessible.

6. **Aucun test framework** : La story 1.5 n'ajoute aucun test unitaire ou d'intégration. Les middlewares de permission, les CRUD users, et les filtres frontend ne sont pas testés.

7. **`required_permission` en dehors du module auth_middleware ne peut pas être réutilisée** : La fonction est privée (`fn`, pas `pub fn`). Si un autre module a besoin de vérifier des permissions (ex: un helper), il ne peut pas réutiliser cette fonction. Pour l'alpha c'est correct, mais à mettre en pub si nécessaire.

8. **La page admin/users utilise `$fetch` sans typage fort** : Les appels API utilisent `$fetch` brut sans schéma Zod. Le typage `User` est déclaré manuellement dans le composant. Si l'API change, le frontend ne détecte pas la divergence.

9. **PATCH /api/users/{id} ne distingue pas "undefined" de "non fourni" pour les champs optionnels** : Avec `#[serde(default)]`, si le client envoie `{"email": null}`, serde le convertit en `Option::None` (car le champ est `Option<String>`). Mais si le client envoie `{"email": ""}`, serde le dé-sérialise en `Some("")`. Le validateur `validate_email` rejette les emails vides, donc c'est géré. ✅ Cependant, un client qui envoie `{"email": null}` pour "ne pas changer l'email" fonctionne correctement.

10. **Pas de gestion d'erreur pour les routes non-existantes sous /api/** : Si une requête arrive sur `/api/nonexistent`, le fallback `Permission::All` est retourné. Le middleware vérifie si l'utilisateur a Permission::All (admin seulement). Un admin verra une 404 (parce que la route n'existe pas dans le router), un non-admin verra une 403. Le 403 est trompeur — l'utilisateur pourrait croire qu'il n'a pas accès à une route qui existe, alors qu'elle n'existe tout simplement pas.

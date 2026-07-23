# Edge Case Hunter — Findings

## Unhandled edge cases identified

### auth_middleware.rs

1. **Path `/api/users` suivi d'un tiret** : `path.starts_with("/api/users")` match aussi `/api/users-extra`. Si une route comme `/api/users-export` est ajoutée plus tard, elle héritera de Permission::ManageUsers au lieu du fallback Permission::All. Correction : utiliser `path == "/api/users" || path.starts_with("/api/users/")`.

2. **Path `/api/settings` avec sous-route** : `path.starts_with("/api/settings")` match aussi `/api/settings-backup`. Même problème qu'au-dessus. Même correction : `path == "/api/settings" || path.starts_with("/api/settings/")`.

3. **Cas du path exact vs starts_with** : Les routes comme `/api/settings` sont déclarées exactes dans le router, mais le middleware utilise `starts_with`. Si une route `/api/settings-advanced` est ajoutée, elle hérite de ManageSettings. Même pattern qu'au-dessus.

4. **Ordre des checks `required_permission`** : `/api/auth/me` est checké AVANT le fallback `starts_with("/api/")` → ✅ géré. Mais si quelqu'un réorganise les ifs plus tard, le fallback `/api/` pourrait masquer les cas spécifiques. Documenter l'ordre comme contract.

5. **Race condition dans create_user** : La vérification `find_by_email` puis `create` n'est pas atomique. Deux requêtes POST simultanées avec le même email peuvent créer deux utilisateurs avec le même email si la première n'a pas fini d'écrire avant la seconde. La contrainte UNIQUE en BDD protège contre la duplication, donc le second INSERT échouera avec une erreur. Le handler catch détecte "UNIQUE" dans l'erreur et retourne 409. ✅ Paradoxalement géré par la BDD, pas par le code applicatif.

6. **PATCH /api/users/{id} avec body vide** : Tous les champs sont `Option` avec `#[serde(default)]`. Body `{}` est valide. Aucun champ n'est modifié. `updated_at` n'est pas mis à jour (car le if let Some ne match pas). C'est correct — update inutile mais pas dangereux.

7. **DELETE sur utilisateur inexistant deux fois** : Premier appel → BDD supprime, retour 200. Deuxième appel → `find_by_id` retourne None → 404. ✅ Géré.

### Frontend — admin/users.vue

8. **Échec de chargement des utilisateurs** : L'erreur est affichée dans un `UAlert`. Il n'y a pas de bouton "Réessayer" après une erreur de chargement. L'utilisateur doit recharger la page.

9. **Modal de création : email déjà pris** : L'API retourne 409. Le catch affiche l'erreur dans `formError`. ✅ Mais le formulaire ne préserve pas les champs déjà saisis (c'est déjà le cas car `form` est lié via `v-model`). ✅

10. **Suppression d'un admin alors qu'il reste un autre admin** : `handleDelete` appelle l'API. Le backend vérifie `admin_count > 1`. Si `admin_count == 2` au moment de la vérification mais un autre admin supprime simultanément, `admin_count` devient 1 pour le second, qui reçoit "LAST_ADMIN". ✅ C'est correct.

11. **Utilisateur non connecté charge /admin/users** : Le middleware `auth` court-circuite vers `/login`. Le middleware `admin` n'est jamais atteint. ✅

12. **Navigation filtrée non réactive** : `usePages()` est appelée une fois au mount du composant. Si l'utilisateur se connecte/déconnecte sans navigation, la navigation ne se met pas à jour. En pratique, après login l'utilisateur est redirigé et la page se re-rend. ✅ Acceptable.

### Frontend — pages.ts

13. **minRole avec un rôle qui n'existe pas** : Si un `definePageMeta` spécifie `minRole: "manager"` mais que ce rôle n'existe pas, l'utilisateur ne verra jamais la page (car `userRole !== minRole` est toujours true). C'est safe (la page est cachée) mais potentiellement confusant.

14. **Catégorie "admin" dans app.config.ts sans vue correspondante** : Si la page admin est filtrée par `minRole`, la catégorie apparaît vide pour les non-admins. Le code `if (!acc[category])` crée la catégorie seulement quand une page y est ajoutée. ✅ Donc la catégorie admin n'apparaît pas du tout pour les non-admins.

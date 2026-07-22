# Blind Hunter — Adversarial Review: Story 1.3 (Auth JWT)

## Findings

1. **Aucun cookie JWT émis par les handlers register/login.** Les AC-1 et AC-2 stipulent qu'un cookie `mboa_session` HTTP-only doit être émis après inscription et connexion. Actuellement, `auth::register` et `auth::login` retournent uniquement un `Json<AuthResponse>` sans header `Set-Cookie`. Le frontend ne peut donc pas obtenir de JWT — toute requête ultérieure sera rejetée par le middleware (401 UNAUTHORIZED).

2. **`encode_token()` défini dans `jwt.rs` mais jamais appelé.** La fonction existe mais n'est utilisée nulle part. Cela confirme le point 1 : le JWT n'est jamais signé ni transmis.

3. **`name` vide dans `/api/auth/me`.** Le handler `me` retourne `name: String::new()` au lieu d'aller chercher le nom en BDD. Le prénom de l'utilisateur est perdu après le login.

4. **`logout()` ne détruit pas le cookie côté serveur.** AC-5 exige que le cookie soit détruit (`Set-Cookie` avec `Max-Age=0`). Actuellement, `logout()` retourne juste `{ message: "Logged out" }` sans header `Set-Cookie`. Le cookie persiste côté navigateur.

5. **Validation email trop laxiste.** La fonction `validate_email` vérifie seulement que le champ contient `@`. Un email comme `"@"` ou `"a@b"` passe la validation. Aucune vérification de format RFC 5322 ou de présence de domaine valide.

6. **Aucune rate limiting / brute-force protection.** Les endpoints `/api/auth/login` et `/api/auth/register` n'ont aucune protection contre les tentatives répétées (rate limiting, account lockout, ou délai progressif).

7. **Clé JWT non persistée entre les redémarrages.** `load_or_generate_jwt_secret()` dans `lib.rs` appelle `generate_secret()` à chaque démarrage, générant une nouvelle clé. Tous les JWT signés avant le redémarrage deviennent invalides. AD-12 spécifie le stockage dans `tauri_plugin_store` mais ce n'est pas implémenté.

8. **Secret JWT visible dans les logs.** En mode DEBUG ou si une erreur survient, le formatage de `jwt_secret` pourrait exposer la clé (via `tracing` ou le débug Display de `Arc<Vec<u8>>`).

9. **Aucune gestion de CORS pour les cookies cross-origin.** Les cookies `SameSite=Lax` ne fonctionneront pas si le frontend est servi depuis un port différent du backend (ce qui arrive en dev avec `scripts/tauri-dev.ts`). Il faudrait `SameSite=None; Secure` avec CORS configuré, mais `Secure` ne fonctionne pas en HTTP.

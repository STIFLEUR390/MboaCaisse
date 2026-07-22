# Acceptance Auditor — Story 1.3 Compliance Review (Updated)

## AC Compliance Status — AFTER FIXES

| AC | Statut | Détail |
|---|---|---|
| **AC-1** Register | ✅ | Argon2 hash, user créé, role caissier, JWT cookie émis (Set-Cookie), 409/422 errors |
| **AC-2** Login | ✅ | Credentials vérifiés, JWT cookie émis, 401 pour erreurs |
| **AC-3** Middleware JWT | ✅ | 401 sans cookie, AuthUser injecté, TOKEN_EXPIRED/INVALID_TOKEN |
| **AC-4** Refresh silencieux | ✅ | should_refresh <1h, X-Token-Refreshed header, nouveau Set-Cookie |
| **AC-5** Logout | ✅ | Set-Cookie: Max-Age=0, cookie détruit |
| **AC-6** Bootstrap admin | ✅ | Premier user → admin, suivants → caissier, seed console |
| **AC-7** Validation entrées | ✅ | Rust backend (email format, password >= 8), serde ignore champs inconnus |
| **AC-8** Page login | ✅ | Formulaire email/password, lien register, erreurs, useAuth(), redirection |
| **AC-9** Page register | ✅ | Formulaire avec confirm, validation frontend, lien login, redirection |

**Résultat : 9/9 ACs satisfaites ✅**

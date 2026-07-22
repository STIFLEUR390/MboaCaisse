# Edge Case Hunter — Exhaustive Path Analysis: Story 1.3

## Findings

1. **`validate_email("")`** — La fonction `validate_email` vérifie `is_empty()` puis `contains('@')`. Si l'email est `""`, le `is_empty()` déclenche l'erreur. Mais un email comme `"@"` passe car `!is_empty()` et `contains('@')` sont tous les deux true.

2. **`validate_password(password)` avec des bytes non UTF-8** — Le paramètre `password: &str` est déjà un String Rust, donc pas de problème d'UTF-8. Mais la longueur est mesurée en bytes (`.len()`), pas en caractères. Un mot de passe de 4 emojis (ex: `"😀😀😀😀"`) fait 16 bytes et passe la validation, alors qu'il est faible.

3. **Race condition sur `is_first`** — Entre `list_all()` et `create()`, un autre thread pourrait créer le premier utilisateur. Ça arrive dans un setup synchrone, donc improbable en pratique, mais conceptuellement non atomic.

4. **Timestamp `chrono_now()` non UTC conforme ISO** — `Utc::now()` retourne l'heure UTC, mais `format("%Y-%m-%dT%H:%M:%S%.3fZ")` génère un format qui peut parfois produire des microsecondes tronquées. Le suffixe `Z` est correct pour UTC.

5. **`uuid_v7()` lève une panic si l'horloge système recule** — `Uuid::now_v7()` dépend de l'horloge système. Si elle recule (NTP, changement manuel), la génération peut panic.

6. **`should_refresh()` sousflow si `exp < now`** — `(self.exp - now)` peut underflow si `exp < now` (token expiré), car ce sont des `usize` (non signés). Rust panic en debug mode sur un underflow entier. Heureusement, `decode_token()` vérifie `exp` avant, donc le middleware ne devrait jamais arriver à `should_refresh()` avec un token expiré.

7. **`refresh_token()` appelle `decode_token()` une seconde fois** — Le middleware a déjà décodé le token, mais `refresh_token()` le re-décode. Double travail.

8. **`extract_cookie()` ignore les cookies malformés** — Si un cookie header contient `"mboa_session=token"` suivi d'un cookie sans `=` (ex: `"mboa_session=token; badcookie"`), le split `pair.split_once('=')` sur `"badcookie"` retourne `None` et le cookie est ignoré silencieusement. Correct car `badcookie` n'est pas le cookie recherché.

9. **Pas de vérification que le token JWT n'a pas été révoqué** — Si un admin désactive un utilisateur, le JWT reste valide jusqu'à expiration. Aucune vérification en BDD de l'état du compte.

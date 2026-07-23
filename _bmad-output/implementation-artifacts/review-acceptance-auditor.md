# Acceptance Auditor — Story 1.5 Review

## Spec: 1-5-roles-permissions-middleware-guard-seed-admin

### AC-1: 4 rôles avec permissions dérivées
❌ / ✅ à vérifier : domain/user.rs préexistant (pas dans le diff)

### AC-2: Admin = Permission::All, toutes routes autorisées
✅ auth_middleware.rs : `Role::has_permission()` → `Permission::All` → match tout → bypass.

### AC-3: Caissier → Sell, ViewReports, ViewOrders. ManageUsers → 403
✅ auth_middleware.rs : `required_permission()` mappe /api/users → ManageUsers. Caissier n'a pas ManageUsers → 403.

### AC-4: Vendeur → ViewOrders, ManageMenu. Sell → 403
✅ required_permission mappe /api/payments → Sell, /api/kitchen → ViewOrders. Vérifié par has_permission().

### AC-5: GestionnaireStock → ManageStock, ViewReports. Sell → 403
✅ required_permission mappe /api/stock → ManageStock, /api/reports → ViewReports.

### AC-6: Seed admin idempotent
✅ Préexistant (db/seed.rs) — pas modifié par cette story.

### AC-7: Middleware role-check — 401/403
❓ auth_middleware vérifie permission APRÈS JWT. 401 si pas de JWT (via unauthorized_response), 403 si permission manquante (via forbidden_response). ✅

### AC-8: CRUD users
✅ GET /api/users → list_users, POST → create_user, PATCH → update_user, DELETE → delete_user. Self-delete protégé. Dernier admin protégé. Validation email/password/role.

### AC-9: Page /admin/users frontend
✅ Vue component avec tableau, modaux, UToast. Middleware auth+admin. ✅

### AC-10: Navigation filtrée par rôle
✅ pages.ts : useAuth() + minRole filtering. settings.vue : minRole: "admin". admin/users : middleware admin.

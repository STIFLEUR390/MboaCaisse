// Nuxt middleware for authentication-based route protection.
//
// Redirects unauthenticated users to /login for protected routes.
// Redirects authenticated users away from /login and /register.
//
// NOTE: On first load, the auth state is not yet initialized (init() is async).
// We allow the request through and let the page handle redirection if needed.

export default defineNuxtRouteMiddleware((to, _from) => {
	const { isAuthenticated, initialized } = useAuth();

	// Public pages that don't require auth
	const publicPages = ["/login", "/register", "/menu"];

	// If auth hasn't initialized yet, allow the request through.
	// The page will check auth on mount and redirect if needed.
	if (!initialized.value) {
		return;
	}

	// If not authenticated and trying to access a protected page
	if (!isAuthenticated.value && !publicPages.includes(to.path)) {
		return navigateTo("/login");
	}

	// If authenticated and trying to access login/register pages
	if (isAuthenticated.value && ["/login", "/register"].includes(to.path)) {
		return navigateTo("/");
	}
});

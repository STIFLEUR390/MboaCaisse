// Nuxt middleware for authentication-based route protection.
//
// Redirects unauthenticated users to /login for protected routes.
// Redirects authenticated users away from /login and /register.

export default defineNuxtRouteMiddleware((to, _from) => {
	const { isAuthenticated } = useAuth();

	// Public pages that don't require auth
	const publicPages = ["/login", "/register", "/menu"];

	// If not authenticated and trying to access a protected page
	if (!isAuthenticated.value && !publicPages.includes(to.path)) {
		return navigateTo("/login");
	}

	// If authenticated and trying to access login/register pages
	if (isAuthenticated.value && ["/login", "/register"].includes(to.path)) {
		return navigateTo("/");
	}
});

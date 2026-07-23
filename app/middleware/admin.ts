// Nuxt middleware for admin role verification.
// Redirects non-admin users to the home page.
//
// NOTE: On first load, the auth state is not yet initialized (init() is async).
// We allow the request through and let the page handle redirection if needed.

export default defineNuxtRouteMiddleware((_to, _from) => {
	const { user, initialized } = useAuth();

	// If auth hasn't initialized yet, allow the request through
	if (!initialized.value) {
		return;
	}

	if (!user.value || user.value.role !== "admin") {
		return navigateTo("/");
	}
});

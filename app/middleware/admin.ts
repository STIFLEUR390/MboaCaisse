// Nuxt middleware for admin role verification.
// Redirects non-admin users to the home page.

export default defineNuxtRouteMiddleware((_to, _from) => {
	const { user } = useAuth();

	if (!user.value || user.value.role !== "admin") {
		return navigateTo("/");
	}
});

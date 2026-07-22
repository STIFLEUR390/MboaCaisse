// Auto-imported composable for authentication state management.
//
// Provides reactive user state, login/register/logout methods, and
// an isAuthenticated computed ref. All API calls use $fetch with
// credentials: 'include' to send the HttpOnly cookie.

interface AuthUser {
	id: string
	email: string
	name: string
	role: string
}

interface AuthResponse {
	id: string
	email: string
	name: string
	role: string
}

interface ApiError {
	error: string
	code: string
}

export const useAuth = () => {
	const user = ref<AuthUser | null>(null);
	const loading = ref(false);
	const error = ref<string | null>(null);

	const isAuthenticated = computed(() => user.value !== null);
	const isAdmin = computed(() => user.value?.role === "admin");

	// Initialize user state from the cookie on mount
	// Since the cookie is HttpOnly, we try to call a protected endpoint
	// to check if we have a valid session. For now, we simply
	// check document.cookie existence as a heuristic.
	async function init() {
		// If a mboa_session cookie exists, try to get the user profile
		if (document.cookie.includes("mboa_session")) {
			try {
				const data = await $fetch<AuthUser>("/api/auth/me", {
					credentials: "include"
				});
				user.value = data;
			} catch {
				// Cookie exists but expired — user stays null
				user.value = null;
			}
		}
	}

	async function login(email: string, password: string): Promise<AuthUser> {
		loading.value = true;
		error.value = null;

		try {
			const data = await $fetch<AuthResponse>("/api/auth/login", {
				method: "POST",
				body: { email, password },
				credentials: "include"
			});

			user.value = {
				id: data.id,
				email: data.email,
				name: data.name,
				role: data.role
			};

			return user.value;
		} catch (err: any) {
			const apiErr = err?.data as ApiError | undefined;
			error.value = apiErr?.error || "Login failed";
			throw new Error(error.value);
		} finally {
			loading.value = false;
		}
	}

	async function register(email: string, password: string, name?: string): Promise<AuthUser> {
		loading.value = true;
		error.value = null;

		try {
			const data = await $fetch<AuthResponse>("/api/auth/register", {
				method: "POST",
				body: { email, password, name },
				credentials: "include"
			});

			user.value = {
				id: data.id,
				email: data.email,
				name: data.name,
				role: data.role
			};

			return user.value;
		} catch (err: any) {
			const apiErr = err?.data as ApiError | undefined;
			error.value = apiErr?.error || "Registration failed";
			throw new Error(error.value);
		} finally {
			loading.value = false;
		}
	}

	async function logout() {
		try {
			await $fetch("/api/auth/logout", {
				method: "POST",
				credentials: "include"
			});
		} catch {
			// Even if the request fails, clear local state
		}
		user.value = null;
		error.value = null;
	}

	// Run init once
	if (import.meta.client) {
		init();
	}

	return {
		user,
		loading,
		error,
		isAuthenticated,
		isAdmin,
		login,
		register,
		logout,
		init
	};
};

<template>
	<div class="flex min-h-screen items-center justify-center bg-(--ui-bg-elevated) px-4">
		<UCard class="w-full max-w-sm">
			<template #header>
				<div class="text-center">
					<h1 class="text-xl font-bold">
						Connexion
					</h1>
					<p class="text-sm text-(--ui-text-muted) mt-1">
						Connectez-vous à MboaCaisse
					</p>
				</div>
			</template>

			<UForm :state="form" class="space-y-4" @submit="handleLogin">
				<UFormField label="Email" name="email" required>
					<UInput
						v-model="form.email"
						type="email"
						placeholder="admin@mboacaisse.local"
						autocomplete="email"
						class="w-full"
					/>
				</UFormField>

				<UFormField label="Mot de passe" name="password" required>
					<UInput
						v-model="form.password"
						type="password"
						placeholder="••••••••"
						autocomplete="current-password"
						class="w-full"
					/>
				</UFormField>

				<UAlert
					v-if="errorMessage"
					color="error"
					:title="errorMessage"
					:icon="false"
				/>

				<UButton
					type="submit"
					color="primary"
					class="w-full"
					:loading="loading"
					:disabled="loading"
				>
					Se connecter
				</UButton>
			</UForm>

			<template #footer>
				<div class="text-center text-sm">
					<NuxtLink to="/register" class="text-(--ui-primary) hover:underline">
						Créer un compte
					</NuxtLink>
				</div>
			</template>
		</UCard>
	</div>
</template>

<script lang="ts" setup>
	definePageMeta({
		name: "login",
		layout: "blank"
	});

	const { login, loading, error } = useAuth();
	const router = useRouter();

	const form = reactive({
		email: "",
		password: ""
	});

	const errorMessage = ref<string | null>(null);

	async function handleLogin() {
		errorMessage.value = null;

		if (!form.email || !form.password) {
			errorMessage.value = "Veuillez remplir tous les champs";
			return;
		}

		try {
			await login(form.email, form.password);
			await router.push("/");
		} catch (err: any) {
			errorMessage.value = err.message || "Email ou mot de passe incorrect";
		}
	}
</script>

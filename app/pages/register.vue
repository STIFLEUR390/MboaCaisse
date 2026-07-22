<template>
	<div class="flex min-h-screen items-center justify-center bg-(--ui-bg-elevated) px-4">
		<UCard class="w-full max-w-sm">
			<template #header>
				<div class="text-center">
					<h1 class="text-xl font-bold">Créer un compte</h1>
					<p class="text-sm text-(--ui-text-muted) mt-1">
						Inscrivez-vous pour accéder à MboaCaisse
					</p>
				</div>
			</template>

			<UForm :state="form" @submit="handleRegister" class="space-y-4">
				<UFormField label="Email" name="email" required>
					<UInput
						v-model="form.email"
						type="email"
						placeholder="email@exemple.com"
						autocomplete="email"
						class="w-full"
					/>
				</UFormField>

				<UFormField label="Nom" name="name" hint="Optionnel">
					<UInput
						v-model="form.name"
						type="text"
						placeholder="Votre nom"
						class="w-full"
					/>
				</UFormField>

				<UFormField label="Mot de passe" name="password" required>
					<UInput
						v-model="form.password"
						type="password"
						placeholder="Minimum 8 caractères"
						autocomplete="new-password"
						class="w-full"
					/>
				</UFormField>

				<UFormField label="Confirmer le mot de passe" name="confirmPassword" required>
					<UInput
						v-model="form.confirmPassword"
						type="password"
						placeholder="••••••••"
						autocomplete="new-password"
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
					Créer mon compte
				</UButton>
			</UForm>

			<template #footer>
				<div class="text-center text-sm">
					<NuxtLink to="/login" class="text-(--ui-primary) hover:underline">
						Déjà un compte ? Se connecter
					</NuxtLink>
				</div>
			</template>
		</UCard>
	</div>
</template>

<script lang="ts" setup>
definePageMeta({
	name: 'register',
	layout: 'blank'
})

const { register, loading } = useAuth()
const router = useRouter()

const form = reactive({
	email: '',
	name: '',
	password: '',
	confirmPassword: ''
})

const errorMessage = ref<string | null>(null)

async function handleRegister() {
	errorMessage.value = null

	if (!form.email || !form.password || !form.confirmPassword) {
		errorMessage.value = 'Veuillez remplir tous les champs obligatoires'
		return
	}

	if (form.password.length < 8) {
		errorMessage.value = 'Le mot de passe doit contenir au moins 8 caractères'
		return
	}

	if (form.password !== form.confirmPassword) {
		errorMessage.value = 'Les mots de passe ne correspondent pas'
		return
	}

	try {
		await register(form.email, form.password, form.name || undefined)
		await router.push('/')
	} catch (err: any) {
		errorMessage.value = err.message || 'Inscription échouée'
	}
}
</script>

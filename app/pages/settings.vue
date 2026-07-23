<template>
	<UContainer>
		<div class="py-8 max-w-2xl mx-auto space-y-8">
			<div>
				<h1 class="text-2xl font-bold font-heading">
					Paramètres
				</h1>
				<p class="text-(--ui-text-muted) mt-1">
					Configuration du serveur et de l'application
				</p>
			</div>

			<!-- Loading state -->
			<div v-if="store.loading" class="flex justify-center py-12">
				<UIcon name="i-lucide-loader-circle" class="size-8 animate-spin text-(--ui-text-muted)" />
			</div>

			<!-- Error alert -->
			<UAlert
				v-if="store.error"
				color="error"
				:title="store.error"
				icon="i-lucide-alert-circle"
				@close="store.error = null"
			/>

			<!-- Settings form — disabled on error to avoid misleading defaults -->
			<UForm v-else-if="!loadError" :state="formState" class="space-y-8" @submit="onSave">
				<!-- Section Serveur -->
				<UCard>
					<template #header>
						<div class="flex items-center gap-2">
							<UIcon name="i-lucide-server" class="size-5" />
							<span class="font-semibold">Serveur</span>
						</div>
					</template>

					<div class="space-y-4">
						<UFormField label="Port" name="port" hint="3000–3099. Redémarrage requis.">
							<UInput
								v-model="formState.port"
								type="number"
								:min="3000"
								:max="3099"
								placeholder="3000"
							/>
						</UFormField>

						<UFormField label="Nom d'hôte mDNS" name="hostname" hint="Ex: mboacaisse → mboacaisse.local. Redémarrage requis.">
							<UInput
								v-model="formState.hostname"
								type="text"
								placeholder="mboacaisse"
							/>
						</UFormField>
					</div>
				</UCard>

				<!-- Section Backup -->
				<UCard>
					<template #header>
						<div class="flex items-center gap-2">
							<UIcon name="i-lucide-database-backup" class="size-5" />
							<span class="font-semibold">Backup</span>
						</div>
					</template>

					<UFormField label="Intervalle de sauvegarde" name="backup_interval_hours" hint="En heures. 1–168 (7 jours).">
						<UInput
							v-model="formState.backup_interval_hours"
							type="number"
							:min="1"
							:max="168"
							placeholder="24"
						/>
					</UFormField>
				</UCard>

				<!-- Section Affichage -->
				<UCard>
					<template #header>
						<div class="flex items-center gap-2">
							<UIcon name="i-lucide-monitor" class="size-5" />
							<span class="font-semibold">Affichage</span>
						</div>
					</template>

					<div class="flex items-center justify-between">
						<div>
							<p class="font-medium">
								Mode headless
							</p>
							<p class="text-sm text-(--ui-text-muted)">
								Démarrer sans fenêtre (icône tray uniquement). Redémarrage requis.
							</p>
						</div>
						<USwitch v-model="formState.headless" />
					</div>
				</UCard>

				<!-- Restart hints -->
				<div v-if="restartRequiredKeys.length > 0" class="rounded-md bg-(--ui-primary) bg-opacity-10 p-4">
					<div class="flex items-start gap-2">
						<UIcon name="i-lucide-info" class="size-5 mt-0.5 shrink-0" />
						<div>
							<p class="font-medium text-sm">
								Redémarrage requis
							</p>
							<p class="text-sm text-(--ui-text-muted)">
								Les modifications suivantes nécessitent un redémarrage :
								{{ restartRequiredKeys.join(', ') }}.
							</p>
						</div>
					</div>
				</div>

				<!-- Actions -->
				<div class="flex items-center gap-3 justify-end">
					<UButton
						color="neutral"
						variant="outline"
						:disabled="store.saving"
						@click="onReset"
					>
						Réinitialiser
					</UButton>
					<UButton
						type="submit"
						color="primary"
						:loading="store.saving"
					>
						Sauvegarder
					</UButton>
				</div>
			</UForm>

			<!-- Retry on load error -->
			<div v-else class="text-center py-12 space-y-4">
				<p class="text-(--ui-text-muted)">Impossible de charger les paramètres.</p>
				<UButton color="primary" variant="outline" @click="initForm">
					Réessayer
				</UButton>
			</div>

			<!-- Restart button (after save) -->
			<div v-if="showRestart" class="flex justify-center">
				<UButton
					color="warning"
					variant="solid"
					icon="i-lucide-rotate-ccw"
					@click="onRestart"
				>
					Redémarrer maintenant
				</UButton>
			</div>
		</div>
	</UContainer>
</template>

<script lang="ts" setup>
	definePageMeta({
		name: "Paramètres",
		minRole: "admin",
		icon: "i-lucide-settings",
		description: "Configuration du système",
		category: "system",
		layout: "default",
		middleware: ["auth", "admin"]
	});

	const store = useSettingsStore();
	const toast = useToast();

	// Track whether initial load failed
	const loadError = ref(false);

	// Local form state (mirrors store.config)
	const formState = reactive({
		port: 3000,
		hostname: "mboacaisse",
		backup_interval_hours: 24,
		headless: false
	});

	const restartRequiredKeys = ref<string[]>([]);
	const showRestart = ref(false);

	// Load settings on mount
	async function initForm() {
		loadError.value = false;
		await store.load();

		if (store.error) {
			loadError.value = true;
			return;
		}

		formState.port = (store.config.port as number) || 3000;
		formState.hostname = (store.config.hostname as string) || "mboacaisse";
		formState.backup_interval_hours = (store.config.backup_interval_hours as number) || 24;
		formState.headless = (store.config.headless as boolean) || false;
	}

	onMounted(initForm);

	async function onSave() {
		const changed: Record<string, any> = {};

		if (formState.port !== store.config.port) {
			changed.port = formState.port;
		}
		if (formState.hostname !== store.config.hostname) {
			changed.hostname = formState.hostname;
		}
		if (formState.backup_interval_hours !== store.config.backup_interval_hours) {
			changed.backup_interval_hours = formState.backup_interval_hours;
		}
		if (formState.headless !== store.config.headless) {
			changed.headless = formState.headless;
		}

		if (Object.keys(changed).length === 0) {
			toast.add({ title: "Aucun changement", color: "neutral" });
			return;
		}

		const result = await store.save(changed);

		// Collect keys that require restart
		restartRequiredKeys.value = result
			.filter((e) => e.requires_restart)
			.map((e) => e.key);

		if (restartRequiredKeys.value.length > 0) {
			showRestart.value = true;
		}

		if (store.error) {
			toast.add({ title: "Erreur", description: store.error, color: "error" });
		} else {
			toast.add({ title: "Paramètres sauvegardés", color: "success" });

			// Update local form state from store
			formState.port = (store.config.port as number) || 3000;
			formState.hostname = (store.config.hostname as string) || "mboacaisse";
			formState.backup_interval_hours = (store.config.backup_interval_hours as number) || 24;
			formState.headless = (store.config.headless as boolean) || false;
		}
	}

	async function onReset() {
		const confirmed = window.confirm(
			"Cette action réinitialisera tous les paramètres aux valeurs par défaut. Les valeurs actuelles seront perdues. Continuer ?"
		);
		if (!confirmed) return;

		await store.reset();

		// Refresh local form
		formState.port = (store.config.port as number) || 3000;
		formState.hostname = (store.config.hostname as string) || "mboacaisse";
		formState.backup_interval_hours = (store.config.backup_interval_hours as number) || 24;
		formState.headless = (store.config.headless as boolean) || false;

		restartRequiredKeys.value = ["port", "hostname", "headless"];
		showRestart.value = true;

		toast.add({ title: "Paramètres réinitialisés", color: "success" });
	}

	async function onRestart() {
		// In Tauri, we can call exit; in browser dev mode, we reload
		try {
			// @ts-ignore — useTauriApp is auto-imported
			await useTauriAppExit(0);
		} catch (err) {
			console.warn("useTauriAppExit not available (dev mode), falling back to reload", err);
			window.location.reload();
		}
	}
</script>

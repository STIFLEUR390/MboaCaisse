<template>
	<div class="space-y-6">
		<div class="flex items-center justify-between">
			<div>
				<h1 class="text-2xl font-bold">
					Gestion des utilisateurs
				</h1>
				<p class="text-sm text-(--ui-text-muted) mt-1">
					Créez, modifiez et supprimez les comptes utilisateurs
				</p>
			</div>
			<UButton color="primary" @click="openCreateModal">
				Ajouter un utilisateur
			</UButton>
		</div>

		<!-- Loading state -->
		<div v-if="loading" class="flex justify-center py-12">
			<UIcon name="i-lucide-loader" class="size-8 animate-spin text-(--ui-text-muted)" />
		</div>

		<!-- Error state -->
		<UAlert
			v-else-if="error"
			color="error"
			:title="error"
			icon="i-lucide-alert-circle"
		/>

		<!-- Users table -->
		<UTable
			v-else
			:columns="columns"
			:data="users"
			class="w-full"
		>
			<template #role-cell="{ row }">
				<UBadge :color="roleColor(row.role)" variant="subtle">
					{{ roleLabel(row.role) }}
				</UBadge>
			</template>

			<template #actions-cell="{ row }">
				<div class="flex gap-2">
					<UButton
						color="neutral"
						variant="ghost"
						size="sm"
						icon="i-lucide-pencil"
						@click="openEditModal(row)"
					/>
					<UButton
						color="error"
						variant="ghost"
						size="sm"
						icon="i-lucide-trash-2"
						:disabled="row.id === currentUserId"
						@click="confirmDelete(row)"
					/>
				</div>
			</template>
		</UTable>

		<!-- Create / Edit Modal -->
		<UModal v-model:open="showModal">
			<template #title>
				{{ editingUser ? "Modifier l'utilisateur" : "Ajouter un utilisateur" }}
			</template>

			<UForm :state="form" class="space-y-4 p-4" @submit="handleSubmit">
				<UFormField label="Email" name="email" required>
					<UInput
						v-model="form.email"
						type="email"
						placeholder="email@exemple.com"
						autocomplete="off"
					/>
				</UFormField>

				<UFormField label="Nom" name="name" hint="Optionnel">
					<UInput
						v-model="form.name"
						placeholder="Nom de l'utilisateur"
					/>
				</UFormField>

				<UFormField
					:label="editingUser ? 'Nouveau mot de passe (laisser vide pour ne pas changer)' : 'Mot de passe'"
					name="password"
					:required="!editingUser"
				>
					<UInput
						v-model="form.password"
						type="password"
						placeholder="Minimum 8 caractères"
						autocomplete="new-password"
					/>
				</UFormField>

				<UFormField label="Rôle" name="role" required>
					<USelect
						v-model="form.role"
						:items="roleOptions"
					/>
				</UFormField>

				<UAlert
					v-if="formError"
					color="error"
					:title="formError"
					:icon="false"
				/>

				<div class="flex justify-end gap-3 pt-2">
					<UButton variant="secondary" @click="closeModal">
						Annuler
					</UButton>
					<UButton type="submit" color="primary" :loading="submitting">
						{{ editingUser ? "Enregistrer" : "Créer" }}
					</UButton>
				</div>
			</UForm>
		</UModal>

		<!-- Delete confirmation -->
		<UModal v-model:open="showDeleteConfirm">
			<template #title>
				Confirmer la suppression
			</template>
			<div class="p-4 space-y-4">
				<p>
					Êtes-vous sûr de vouloir supprimer l'utilisateur
					<strong>{{ deletingUser?.email }}</strong> ?
				</p>
				<p v-if="deletingUser?.role === 'admin'" class="text-sm text-(--ui-text-muted)">
					Cet utilisateur est un admin. Vérifiez qu'il reste au moins un admin après suppression.
				</p>
				<div class="flex justify-end gap-3">
					<UButton variant="secondary" @click="showDeleteConfirm = false">
						Annuler
					</UButton>
					<UButton color="error" :loading="submitting" @click="handleDelete">
						Supprimer
					</UButton>
				</div>
			</div>
		</UModal>
	</div>
</template>

<script lang="ts" setup>
	definePageMeta({
		name: "admin-users",
		layout: "default",
		middleware: ["auth", "admin"]
	});

	interface User {
		id: string
		email: string
		name: string
		role: string
		created_at: string
	}

	const columns = [
		{ key: "email", label: "Email" },
		{ key: "name", label: "Nom" },
		{ key: "role", label: "Rôle" },
		{ key: "created_at", label: "Créé le" },
		{ key: "actions", label: "Actions" }
	];

	const roleOptions = [
		{ value: "admin", label: "Administrateur" },
		{ value: "caissier", label: "Caissier" },
		{ value: "vendeur", label: "Vendeur" },
		{ value: "gestionnaire_stock", label: "Gestionnaire de stock" }
	];

	const { user: currentUser } = useAuth();
	const currentUserId = computed(() => currentUser.value?.id);
	const toast = useToast();

	const users = ref<User[]>([]);
	const loading = ref(true);
	const error = ref<string | null>(null);
	const showModal = ref(false);
	const showDeleteConfirm = ref(false);
	const editingUser = ref<User | null>(null);
	const deletingUser = ref<User | null>(null);
	const submitting = ref(false);
	const formError = ref<string | null>(null);

	const form = reactive({
		email: "",
		name: "",
		password: "",
		role: "caissier"
	});

	function roleColor(role: string): string {
		switch (role) {
		case "admin": return "info";
		case "caissier": return "success";
		case "vendeur": return "warning";
		case "gestionnaire_stock": return "neutral";
		default: return "neutral";
		}
	}

	function roleLabel(role: string): string {
		const option = roleOptions.find((o) => o.value === role);
		return option?.label || role;
	}

	async function loadUsers() {
		loading.value = true;
		error.value = null;
		try {
			const data = await $fetch<User[]>("/api/users", { credentials: "include" });
			users.value = data;
		} catch (err: any) {
			error.value = err?.data?.error || "Erreur lors du chargement des utilisateurs";
		} finally {
			loading.value = false;
		}
	}

	function openCreateModal() {
		editingUser.value = null;
		form.email = "";
		form.name = "";
		form.password = "";
		form.role = "caissier";
		formError.value = null;
		showModal.value = true;
	}

	function openEditModal(user: User) {
		editingUser.value = user;
		form.email = user.email;
		form.name = user.name;
		form.password = "";
		form.role = user.role;
		formError.value = null;
		showModal.value = true;
	}

	function closeModal() {
		showModal.value = false;
		editingUser.value = null;
	}

	async function handleSubmit() {
		formError.value = null;

		if (!form.email) {
			formError.value = "L'email est requis";
			return;
		}

		if (!editingUser.value && !form.password) {
			formError.value = "Le mot de passe est requis";
			return;
		}

		if (form.password && form.password.length < 8) {
			formError.value = "Le mot de passe doit contenir au moins 8 caractères";
			return;
		}

		submitting.value = true;
		try {
			if (editingUser.value) {
				const body: Record<string, any> = {};
				if (form.email !== editingUser.value.email) body.email = form.email;
				if (form.name !== editingUser.value.name) body.name = form.name;
				if (form.password) body.password = form.password;
				if (form.role !== editingUser.value.role) body.role = form.role;

				await $fetch(`/api/users/${editingUser.value.id}`, {
					method: "PATCH",
					body,
					credentials: "include"
				});
				toast.add({ title: "Utilisateur modifié", color: "success" });
			} else {
				await $fetch("/api/users", {
					method: "POST",
					body: {
						email: form.email,
						password: form.password,
						name: form.name || undefined,
						role: form.role
					},
					credentials: "include"
				});
				toast.add({ title: "Utilisateur créé", color: "success" });
			}
			closeModal();
			await loadUsers();
		} catch (err: any) {
			formError.value = err?.data?.error || "Une erreur est survenue";
		} finally {
			submitting.value = false;
		}
	}

	function confirmDelete(user: User) {
		deletingUser.value = user;
		showDeleteConfirm.value = true;
	}

	async function handleDelete() {
		if (!deletingUser.value) return;
		submitting.value = true;
		try {
			await $fetch(`/api/users/${deletingUser.value.id}`, {
				method: "DELETE",
				credentials: "include"
			});
			toast.add({ title: "Utilisateur supprimé", color: "success" });
			showDeleteConfirm.value = false;
			deletingUser.value = null;
			await loadUsers();
		} catch (err: any) {
			toast.add({
				title: err?.data?.error || "Erreur lors de la suppression",
				color: "error"
			});
		} finally {
			submitting.value = false;
		}
	}

	// Load on mount
	onMounted(() => {
		loadUsers();
	});
</script>

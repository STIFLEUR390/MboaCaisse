<template>
	<div
		class="flex flex-col h-screen bg-(--ui-bg) overflow-hidden"
	>
		<!-- Header -->
		<header class="flex items-center justify-between px-6 py-4 border-b border-(--ui-border) bg-(--ui-bg-elevated) shrink-0">
			<div>
				<h1 class="text-2xl font-bold font-heading">
					Cuisine
				</h1>
				<p class="text-sm text-(--ui-text-muted)">
					{{ inPreparationCount }} en preparation &middot; {{ readyCount }} pretes
				</p>
			</div>
			<div class="flex items-center gap-2">
				<span class="text-xs text-(--ui-text-muted)">{{ lastUpdate }}</span>
				<UButton
					variant="ghost" color="neutral" icon="i-lucide-refresh-cw"
					size="sm" :loading="pending" @click="refresh"
				/>
			</div>
		</header>

		<!-- Loading state (initial only) -->
		<div v-if="pending && !data" class="flex-1 flex items-center justify-center">
			<UIcon name="i-lucide-loader-circle" class="size-8 animate-spin text-(--ui-text-muted)" />
		</div>

		<!-- Error state -->
		<div v-else-if="showError" class="flex-1 flex flex-col items-center justify-center gap-4">
			<UIcon name="i-lucide-wifi-off" class="size-12 text-(--ui-error)" />
			<p class="text-lg font-medium">
				Connexion perdue
			</p>
			<p class="text-sm text-(--ui-text-muted)">
				Impossible de charger les commandes cuisine
			</p>
			<UButton color="primary" variant="outline" @click="retry">
				Reessayer
			</UButton>
		</div>

		<!-- Empty state -->
		<div
			v-else-if="inPreparationCount === 0 && readyCount === 0 && !pending"
			class="flex-1 flex flex-col items-center justify-center gap-4"
		>
			<UIcon name="i-lucide-chef-hat" class="size-16 text-(--ui-text-muted)" />
			<p class="text-xl font-medium text-(--ui-text-muted)">
				Aucune commande en cours
			</p>
			<p class="text-sm text-(--ui-text-muted)">
				Les nouvelles commandes apparaitront ici
			</p>
		</div>

		<!-- Kitchen columns -->
		<div v-else class="flex-1 flex flex-col lg:flex-row overflow-hidden">
			<!-- En preparation -->
			<div class="flex-1 flex flex-col overflow-hidden border-r border-(--ui-border)">
				<div class="px-4 py-3 bg-(--ui-bg-elevated) border-b border-(--ui-border) shrink-0">
					<h2 class="font-semibold text-base flex items-center gap-2">
						<span class="size-2 rounded-full bg-(--ui-primary) inline-block" />
						En preparation
						<span class="text-sm font-normal text-(--ui-text-muted)">({{ inPreparationCount }})</span>
					</h2>
				</div>
				<div class="flex-1 overflow-y-auto p-4 space-y-3">
					<div
						v-for="order in inPreparationOrders" :key="order.id"
						class="bg-(--ui-bg) rounded-lg border border-(--ui-border) border-l-4 border-l-(--ui-primary) p-4 shadow-sm"
					>
						<OrderCardContent :order="order" />
						<div class="mt-3 pt-3 border-t border-(--ui-border) flex justify-end">
							<UButton color="primary" size="sm" @click="markReady(order.id)">
								Prete
							</UButton>
						</div>
					</div>
				</div>
			</div>

			<!-- Pretes -->
			<div class="flex-1 flex flex-col overflow-hidden">
				<div class="px-4 py-3 bg-(--ui-bg-elevated) border-b border-(--ui-border) shrink-0">
					<h2 class="font-semibold text-base flex items-center gap-2">
						<span class="size-2 rounded-full bg-(--ui-success) inline-block" />
						Pretes
						<span class="text-sm font-normal text-(--ui-text-muted)">({{ readyCount }})</span>
					</h2>
				</div>
				<div class="flex-1 overflow-y-auto p-4 space-y-3 bg-(--ui-success)/5">
					<div
						v-for="order in readyOrders" :key="order.id"
						class="bg-(--ui-bg) rounded-lg border border-(--ui-border) border-l-4 border-l-(--ui-success) p-4 shadow-sm"
					>
						<OrderCardContent :order="order" />
						<div class="mt-3 pt-3 border-t border-(--ui-border) flex justify-end">
							<UButton color="success" size="sm" @click="markDelivered(order.id)">
								Servie
							</UButton>
						</div>
					</div>
				</div>
			</div>
		</div>
	</div>
</template>

<script lang="ts" setup>
	definePageMeta({
		name: "Cuisine",
		icon: "i-lucide-cooking-pot",
		description: "Ecran cuisine - commandes en preparation",
		category: "interface",
		layout: "blank",
		middleware: ["auth"]
	});

	const toast = useToast();

	// --- Types ---
	interface KitchenItem {
		product_id: string
		name: string
		quantity: number
		unit_price: number
		notes: string | null
	}

	interface KitchenOrder {
		id: string
		table_id: string | null
		client_id: string | null
		status: string
		total: number
		created_at: string
		updated_at: string
		items: KitchenItem[]
		elapsed_min: number
	}

	interface KitchenData {
		in_preparation: KitchenOrder[]
		ready: KitchenOrder[]
	}

	// --- Fetch with polling ---
	const { data, refresh, pending, error } = useFetch<KitchenData>("/api/kitchen/orders", {
		server: false
	});

	// Derived state
	const inPreparationOrders = computed(() => data.value?.in_preparation ?? []);
	const readyOrders = computed(() => data.value?.ready ?? []);
	const inPreparationCount = computed(() => inPreparationOrders.value.length);
	const readyCount = computed(() => readyOrders.value.length);

	// Track initial load to avoid notification on first fetch
	const initialLoad = ref(true);

	// Detect new orders for notification sound
	watch(inPreparationCount, (newCount: number, oldCount: number) => {
		if (initialLoad.value) {
			initialLoad.value = false;
			return;
		}
		if (newCount > oldCount && !document.hidden) {
			playNotificationSound();
		}
	});

	// Last update timestamp
	const lastUpdate = computed(() => {
		const now = new Date();
		return now.toLocaleTimeString("fr-FR", { hour: "2-digit", minute: "2-digit", second: "2-digit" });
	});

	// Polling interval 5s (AD-14)
	let pollInterval: ReturnType<typeof setInterval> | null = null;
	onMounted(() => {
		pollInterval = setInterval(() => {
			if (!document.hidden) {
				refresh();
			}
		}, 5000);
	});
	onUnmounted(() => {
		if (pollInterval) clearInterval(pollInterval);
	});

	// Consecutive error tracking
	const failCount = ref(0);
	const showError = ref(false);

	watch(error, (err) => {
		if (err) {
			failCount.value++;
			if (failCount.value >= 3) {
				showError.value = true;
			}
		} else {
			failCount.value = 0;
			showError.value = false;
		}
	});

	function retry() {
		showError.value = false;
		failCount.value = 0;
		refresh();
	}

	// --- Actions ---
	async function markReady(orderId: string) {
		try {
			await $fetch(`/api/orders/${orderId}/status`, {
				method: "PATCH",
				body: { status: "ready" }
			});
			refresh();
		} catch (err) {
			console.error("Failed to mark order as ready:", err);
			toast.add({ title: "Erreur", description: "Impossible de marquer la commande comme prete", color: "error" });
		}
	}

	async function markDelivered(orderId: string) {
		try {
			await $fetch(`/api/orders/${orderId}/status`, {
				method: "PATCH",
				body: { status: "delivered" }
			});
			refresh();
		} catch (err) {
			console.error("Failed to mark order as delivered:", err);
			toast.add({ title: "Erreur", description: "Impossible de marquer la commande comme servie", color: "error" });
		}
	}

	// --- Notification sound (Web Audio API) ---
	let audioCtx: AudioContext | null = null;

	function getAudioCtx(): AudioContext {
		if (!audioCtx) {
			audioCtx = new (window.AudioContext || (window as any).webkitAudioContext)();
		}
		return audioCtx;
	}

	function playNotificationSound() {
		if (document.hidden) return;
		try {
			const ctx = getAudioCtx();
			const osc = ctx.createOscillator();
			const gain = ctx.createGain();
			osc.connect(gain);
			gain.connect(ctx.destination);
			osc.frequency.value = 880;
			gain.gain.value = 0.3;
			gain.gain.exponentialRampToValueAtTime(0.001, ctx.currentTime + 0.3);
			osc.start();
			osc.stop(ctx.currentTime + 0.3);
		} catch (err) {
			console.warn("Audio notification failed:", err);
		}
	}
</script>

<style scoped>
	.overflow-y-auto::-webkit-scrollbar { width: 6px; }
	.overflow-y-auto::-webkit-scrollbar-thumb { background: var(--ui-border); border-radius: 3px; }
</style>

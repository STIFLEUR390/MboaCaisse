<template>
	<div>
		<div class="flex items-start justify-between mb-2">
			<div>
				<p class="font-semibold text-base">
					{{ order.table_id ? `Table ${order.table_id}` : 'A emporter' }}
				</p>
				<p class="text-xs text-(--ui-text-muted)">
					{{ order.id.slice(0, 8) }}
				</p>
			</div>
			<div :class="elapsedClass" class="text-sm font-medium shrink-0">
				{{ formatElapsed(order.elapsed_min) }}
			</div>
		</div>

		<div class="space-y-1">
			<div
				v-for="item in order.items" :key="item.product_id"
				class="flex items-center gap-2 text-sm"
			>
				<span class="font-medium">{{ item.name }}</span>
				<span class="text-(--ui-text-muted)">x{{ item.quantity }}</span>
				<span v-if="item.notes" class="text-xs text-(--ui-text-muted) italic">({{ item.notes }})</span>
			</div>
		</div>

		<p class="text-sm font-semibold mt-2">
			{{ order.total.toLocaleString('fr-FR') }} FCFA
		</p>
	</div>
</template>

<script lang="ts" setup>
	const props = defineProps<{
		order: {
			id: string
			table_id: string | null
			client_id: string | null
			status: string
			total: number
			created_at: string
			updated_at: string
			items: Array<{
				product_id: string
				name: string
				quantity: number
				unit_price: number
				notes: string | null
			}>
			elapsed_min: number
		}
	}>();

	const elapsedClass = computed(() => {
		if (props.order.elapsed_min > 10) return "text-(--ui-error)";
		if (props.order.elapsed_min > 5) return "text-(--ui-warning)";
		return "text-(--ui-text-muted)";
	});

	function formatElapsed(min: number): string {
		if (min < 1) return "< 1 min";
		if (min < 60) return `${min} min`;
		const h = Math.floor(min / 60);
		const m = min % 60;
		return `${h}h${m > 0 ? `${m}min` : ""}`;
	}
</script>

// ! Settings store — Pinia bridge between frontend UI and REST API.
// !
// ! AD-12: All system config goes through tauri_plugin_store (via API).
// ! This store syncs settings between the frontend and the Rust backend.
// ! Story 1.4 — AC-4.

export interface SettingEntry {
	key: string
	value: number | string | boolean
	requires_restart?: boolean
}

interface SettingsResponse {
	settings: SettingEntry[]
}

interface SettingsState {
	config: Record<string, number | string | boolean>
	loading: boolean
	saving: boolean
	error: string | null
}

export const useSettingsStore = defineStore("settings", () => {
	const state = reactive<SettingsState>({
		config: {
			port: 3000,
			hostname: "mboacaisse",
			backup_interval_hours: 24,
			headless: false
		},
		loading: false,
		saving: false,
		error: null
	});

	/** Load all settings from the API. */
	async function load() {
		state.loading = true;
		state.error = null;

		try {
			const res = await $fetch<SettingsResponse>("/api/settings", {
				credentials: "include"
			});

			for (const entry of res.settings) {
				state.config[entry.key] = entry.value as any;
			}
		} catch (err: any) {
			state.error = err?.data?.error || err?.message || "Failed to load settings";
			console.error("Settings load error:", state.error);
		} finally {
			state.loading = false;
		}
	}

	/**
	 * Save partial settings to the API.
	 *  Returns the response entries (with requires_restart flags).
	 */
	async function save(partial: Partial<Record<string, number | string | boolean>>): Promise<SettingEntry[]> {
		state.saving = true;
		state.error = null;

		try {
			const res = await $fetch<SettingsResponse>("/api/settings", {
				method: "PATCH",
				body: partial,
				credentials: "include"
			});

			// Update local state from response
			for (const entry of res.settings) {
				state.config[entry.key] = entry.value as any;
			}

			return res.settings;
		} catch (err: any) {
			state.error = err?.data?.error || err?.message || "Failed to save settings";
			console.error("Settings save error:", state.error);
			return [];
		} finally {
			state.saving = false;
		}
	}

	/** Reset all settings to defaults. */
	async function reset() {
		state.loading = true;
		state.error = null;

		try {
			await $fetch("/api/settings", {
				method: "DELETE",
				credentials: "include"
			});

			// Reload defaults from server
			await load();
		} catch (err: any) {
			state.error = err?.data?.error || err?.message || "Failed to reset settings";
			console.error("Settings reset error:", state.error);
		} finally {
			state.loading = false;
		}
	}

	return {
		config: computed(() => state.config),
		loading: computed(() => state.loading),
		saving: computed(() => state.saving),
		error: computed(() => state.error),
		load,
		save,
		reset
	};
});

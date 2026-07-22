# Async Components with keep-alive Ref Issues

## Rule

When using `<keep-alive>`, `<component>`, and `defineAsyncComponent` together, be aware that template refs can become undefined when the component is re-activated after being deactivated.

## Why This Matters

This is a known Vue issue where the ref binding works correctly on first activation but becomes undefined on subsequent activations. This can cause runtime errors when trying to access component methods or properties through refs.

## Problem Scenario

```vue
<template>
	<keep-alive>
		<component :is="currentComponent" ref="widgetRef" />
	</keep-alive>
</template>

<script setup>
	import { defineAsyncComponent, ref } from "vue";

	const AsyncWidget = defineAsyncComponent(() =>
		import("./Widget.vue")
	);

	const currentComponent = ref(AsyncWidget);
	const widgetRef = ref(null);

	function callWidgetMethod() {
		// May be undefined after component reactivation!
		widgetRef.value?.doSomething();
	}
</script>
```

## Workarounds

### Option 1: Use onActivated to re-establish ref access

```vue
<script setup>
	import { defineAsyncComponent, nextTick, onActivated, ref } from "vue";

	const AsyncWidget = defineAsyncComponent(() =>
		import("./Widget.vue")
	);

	const currentComponent = ref(AsyncWidget);
	const widgetRef = ref(null);

	// Use a computed or method that waits for ref to be available
	async function callWidgetMethod() {
		await nextTick();
		if (widgetRef.value) {
			widgetRef.value.doSomething();
		}
	}
</script>
```

### Option 2: Avoid mixing all three patterns

If possible, use one of these alternatives:

```vue
<!-- Option A: Don't use keep-alive with async components -->
<template>
	<component :is="currentComponent" ref="widgetRef" />
</template>

<!-- Option B: Use static component with keep-alive -->
<template>
  <keep-alive>
    <component :is="Widget" ref="widgetRef" />
  </keep-alive>
</template>

<script setup>
	import Widget from "./Widget.vue"; // Regular import
</script>
```

### Option 3: Use provide/inject instead of refs

```vue
<!-- Parent.vue -->
<script setup>
	import { inject, provide, ref } from "vue";
</script>

<!-- Widget.vue (async component) -->
<script setup>
	const sharedState = ref({ /* shared data */ });
	provide("widgetState", sharedState);
	const widgetState = inject("widgetState");
</script>
```

## Key Points

1. This is a known issue when combining `<keep-alive>`, `<component :is>`, and `defineAsyncComponent`
2. Refs may become undefined after component deactivation/reactivation
3. Use `nextTick` and null checks when accessing refs
4. Consider alternative patterns like provide/inject for cross-component communication
5. Test thoroughly when using this combination

## References

- [Vue.js GitHub Discussion #11334](https://github.com/orgs/vuejs/discussions/11334)
- [Vue.js Async Components Documentation](https://vuejs.org/guide/components/async)

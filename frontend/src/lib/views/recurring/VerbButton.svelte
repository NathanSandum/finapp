<script lang="ts">
	import type { VerbConfig } from '$lib/views/entries/verbs';

	let {
		v,
		active,
		onclick
	}: {
		v: VerbConfig;
		active: boolean;
		onclick: () => void;
	} = $props();
</script>

<button
	type="button"
	{onclick}
	class="flex-1 px-3 py-1.5 text-xs font-medium uppercase tracking-[0.06em] transition-colors {active
		? 'bg-surface-card text-ink shadow-sm'
		: 'text-ink-muted hover:text-ink-body'}"
>
	{v.label}
</button>

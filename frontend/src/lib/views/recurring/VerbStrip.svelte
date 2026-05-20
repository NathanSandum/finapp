<script lang="ts">
	import { VERBS, type VerbId } from '$lib/views/entries/verbs';

	let { verb = $bindable() }: { verb: VerbId } = $props();
</script>

<div class="flex flex-wrap gap-1 border border-border bg-surface-sunken p-1">
	{#each VERBS as v (v.id)}
		<button
			type="button"
			onclick={() => (verb = v.id)}
			class="flex-1 px-3 py-1.5 text-xs font-medium uppercase tracking-[0.06em] transition-colors {v.id ===
			verb
				? 'bg-surface-card text-ink shadow-sm'
				: 'text-ink-muted hover:text-ink-body'}"
		>
			{v.label}
		</button>
	{/each}
</div>

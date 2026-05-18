<script lang="ts">
	import type { Snippet } from 'svelte';

	let {
		onclick,
		disabled = false,
		tone = 'indigo',
		size = 'sm',
		class: extra = '',
		children
	}: {
		onclick?: (event: MouseEvent) => void;
		disabled?: boolean;
		tone?: 'indigo' | 'negative';
		size?: 'xs' | 'sm';
		class?: string;
		children: Snippet;
	} = $props();

	const sizeClass = $derived(
		size === 'xs' ? 'px-1.5 py-0.5 text-[10px]' : 'px-2 py-1 text-[11px]'
	);
	const toneClass = $derived(
		tone === 'negative'
			? 'hover:border-negative-strong hover:text-negative-strong'
			: 'hover:border-indigo-500 hover:text-indigo-500'
	);
</script>

<button
	type="button"
	{onclick}
	{disabled}
	class="inline-flex items-center gap-1 border border-border bg-surface-card font-medium uppercase tracking-[0.06em] text-ink-body transition-colors disabled:opacity-50 {sizeClass} {toneClass} {extra}"
>
	{@render children()}
</button>

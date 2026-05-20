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
		size === 'xs' ? 'px-2 py-0.5 text-[10px]' : 'px-2.5 py-1 text-[11px]'
	);
	const toneClass = $derived(
		tone === 'negative'
			? 'hover:border-negative hover:text-negative'
			: 'hover:border-ink hover:text-ink'
	);
</script>

<button
	type="button"
	{onclick}
	{disabled}
	class="inline-flex items-center gap-1 rounded-none border border-border bg-transparent font-medium uppercase tracking-[0.12em] text-ink-body transition-colors disabled:opacity-50 {sizeClass} {toneClass} {extra}"
>
	{@render children()}
</button>

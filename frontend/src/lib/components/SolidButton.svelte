<script lang="ts">
	import type { Snippet } from 'svelte';

	let {
		onclick,
		type = 'button',
		disabled = false,
		size = 'md',
		class: extra = '',
		children
	}: {
		onclick?: (event: MouseEvent) => void;
		type?: 'button' | 'submit';
		disabled?: boolean;
		size?: 'sm' | 'md';
		class?: string;
		children: Snippet;
	} = $props();

	const sizeClass = $derived(
		size === 'sm'
			? 'px-3 py-1 text-[11px] uppercase tracking-[0.14em]'
			: 'px-4 py-2 text-xs uppercase tracking-[0.14em]'
	);
</script>

<button
	{type}
	{onclick}
	{disabled}
	class="rounded-none border border-ink bg-ink font-medium text-ink-inverse transition hover:bg-ink/85 disabled:opacity-40 {sizeClass} {extra}"
>
	{@render children()}
</button>

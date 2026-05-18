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
			? 'px-3 py-1.5 text-xs uppercase tracking-[0.08em]'
			: 'px-4 py-2 text-sm'
	);
</script>

<button
	{type}
	{onclick}
	{disabled}
	class="bg-surface-inverse font-medium text-ink-inverse transition-colors hover:bg-surface-inverse-hover disabled:opacity-50 {sizeClass} {extra}"
>
	{@render children()}
</button>

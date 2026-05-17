<script lang="ts">
	import type { Snippet } from 'svelte';

	let {
		direction,
		onClose,
		children
	}: {
		direction: 'up' | 'down';
		onClose: () => void;
		children: Snippet;
	} = $props();

	let el = $state<HTMLDivElement | null>(null);

	$effect(() => {
		const anchor = el?.parentElement;
		if (!anchor) return;
		function onPointerDown(event: PointerEvent) {
			if (!anchor!.contains(event.target as Node)) onClose();
		}
		window.addEventListener('pointerdown', onPointerDown);
		return () => window.removeEventListener('pointerdown', onPointerDown);
	});

	const positionClass = $derived(direction === 'down' ? 'top-full mt-1' : 'bottom-full mb-1');
	const dropShadow = $derived(
		direction === 'down'
			? '0 18px 40px -8px rgba(0,0,0,0.6)'
			: '0 -18px 40px -8px rgba(0,0,0,0.6)'
	);
	const edgeHighlight = $derived(
		direction === 'down'
			? 'inset 0 1px 0 oklch(60% 0.04 260)'
			: 'inset 0 -1px 0 oklch(60% 0.04 260)'
	);
</script>

<div
	bind:this={el}
	class="absolute -inset-x-2 z-20 {positionClass} bg-surface-overlay shadow-2xl shadow-black/60 ring-1 ring-inset ring-ink-inverse/25"
	style="box-shadow: {edgeHighlight}, {dropShadow};"
	role="menu"
>
	{@render children()}
</div>

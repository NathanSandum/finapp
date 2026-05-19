<script lang="ts">
	import { untrack } from 'svelte';
	import { prefersReducedMotion } from '$lib/utils/motion';
	import NumCell from './NumCell.svelte';

	let { text }: { text: string } = $props();

	type Mode = 'up' | 'down' | 'none';
	type Cell = { key: number; before: string; after: string; mode: Mode };

	function parseHead(t: string) {
		const match = t.match(/^([+\-\u2212]?)(\$?)(.*)$/);
		return {
			head: (match?.[1] ?? '') + (match?.[2] ?? ''),
			body: match?.[3] ?? t
		};
	}

	function parseNum(t: string): number {
		const cleaned = t.replace(/\u2212/g, '-').replace(/[^0-9.\-]/g, '');
		const n = parseFloat(cleaned);
		return isNaN(n) ? 0 : n;
	}

	const parsed = $derived(parseHead(text));

	let nextKey = 1;
	let prevBody = untrack(() => parseHead(text).body);
	let prevNum = untrack(() => parseNum(text));
	let cells = $state<Cell[]>(
		untrack(() =>
			parseHead(text)
				.body.split('')
				.map((ch) => ({
					key: nextKey++,
					before: ch,
					after: ch,
					mode: 'none' as Mode
				}))
		)
	);

	$effect(() => {
		const body = parsed.body;
		if (body === prevBody) return;
		const num = parseNum(text);
		const reduce = prefersReducedMotion();
		const dir: Mode = reduce ? 'none' : num >= prevNum ? 'up' : 'down';
		const oldChars = prevBody.split('');
		const newChars = body.split('');
		const oldLen = oldChars.length;
		const newLen = newChars.length;
		cells = newChars.map((ch, j) => {
			const rightOffset = newLen - 1 - j;
			const oldIdx = oldLen - 1 - rightOffset;
			const before = oldIdx >= 0 ? oldChars[oldIdx] : ch;
			const isNew = oldIdx < 0;
			const mode: Mode = isNew || before === ch ? 'none' : dir;
			return { key: nextKey++, before, after: ch, mode };
		});
		prevBody = body;
		prevNum = num;
	});
</script>

<span class="num"
	><span class="copy-text">{text}</span><span class="visual" aria-hidden="true"
		>{parsed.head}{#each cells as cell, i (cells.length - 1 - i)}{@const survivor =
				i === 0 ||
				Math.floor((i * 3) / cells.length) > Math.floor(((i - 1) * 3) / cells.length)}<NumCell
				{cell}
				{survivor}
			/>{/each}</span
	></span
>

<style>
	.num {
		position: relative;
		display: inline-block;
		white-space: nowrap;
	}
	.copy-text {
		color: transparent;
		user-select: text;
		white-space: nowrap;
	}
	.visual {
		position: absolute;
		inset: 0;
		user-select: none;
		pointer-events: none;
		white-space: nowrap;
	}
</style>

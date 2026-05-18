<script lang="ts">
	import { untrack } from 'svelte';
	import { prefersReducedMotion } from '$lib/utils/motion';
	import NumCells from './NumCells.svelte';

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

<span class="num">{parsed.head}<NumCells {cells} /></span>

<style>
	.num {
		white-space: nowrap;
	}
</style>

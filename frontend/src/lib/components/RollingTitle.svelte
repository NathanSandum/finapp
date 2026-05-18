<script lang="ts">
	let {
		value,
		position = 0
	}: {
		value: string;
		position?: number;
	} = $props();

	type Mode = 'up' | 'down' | 'none';
	type Step = { key: number; before: string; after: string; mode: Mode };

	let step = $state<Step>({ key: 0, before: '', after: value, mode: 'none' });
	let prevValue = value;
	let prevPosition = position;
	let nextKey = 1;

	$effect(() => {
		const curr = value;
		const pos = position;
		if (curr === prevValue) {
			prevPosition = pos;
			return;
		}
		const mode: Mode = pos >= prevPosition ? 'up' : 'down';
		step = { key: nextKey++, before: prevValue, after: curr, mode };
		prevValue = curr;
		prevPosition = pos;
	});
</script>

<span class="rolling-title" aria-label={value}>
	{#key step.key}
		<span class="stage mode-{step.mode}">
			<span class="cell before">{step.before || '\u00A0'}</span>
			<span class="cell after">{step.after || '\u00A0'}</span>
		</span>
	{/key}
</span>

<style>
	.rolling-title {
		display: inline-block;
		position: relative;
		line-height: 1.1;
		height: 1.1em;
		overflow: hidden;
		vertical-align: top;
	}
	.stage {
		position: absolute;
		inset: 0;
	}
	.cell {
		position: absolute;
		inset: 0;
		display: block;
		line-height: 1.1;
		white-space: nowrap;
	}

	.stage.mode-none .before {
		display: none;
	}

	.stage.mode-up .before {
		animation: slide-out-up 460ms cubic-bezier(0.22, 1, 0.36, 1) both;
	}
	.stage.mode-up .after {
		animation: slide-in-up 460ms cubic-bezier(0.22, 1, 0.36, 1) both;
	}
	.stage.mode-down .before {
		animation: slide-out-down 460ms cubic-bezier(0.22, 1, 0.36, 1) both;
	}
	.stage.mode-down .after {
		animation: slide-in-down 460ms cubic-bezier(0.22, 1, 0.36, 1) both;
	}

	@keyframes slide-in-up {
		from {
			transform: translateY(100%);
			opacity: 0;
		}
		to {
			transform: translateY(0);
			opacity: 1;
		}
	}
	@keyframes slide-out-up {
		from {
			transform: translateY(0);
			opacity: 1;
		}
		to {
			transform: translateY(-100%);
			opacity: 0;
		}
	}
	@keyframes slide-in-down {
		from {
			transform: translateY(-100%);
			opacity: 0;
		}
		to {
			transform: translateY(0);
			opacity: 1;
		}
	}
	@keyframes slide-out-down {
		from {
			transform: translateY(0);
			opacity: 1;
		}
		to {
			transform: translateY(100%);
			opacity: 0;
		}
	}

	@media (prefers-reduced-motion: reduce) {
		.cell {
			animation: none !important;
		}
	}
</style>

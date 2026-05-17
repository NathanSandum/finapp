<script lang="ts">
	let eye1 = $state(false);
	let eye2 = $state(false);
	let eye3 = $state(false);
	let eye4 = $state(false);
	let sw1 = $state(false);
	let sw2 = $state(false);
	let sw3 = $state(false);
	let sw4 = $state(false);
</script>

<!-- Eye variants. All use the current half-fill micro pill switch. -->
{#snippet eyeBaseline(on: boolean)}
	<svg class="h-4 w-4" viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="1.5" aria-hidden="true">
		{#if on}
			<path d="M2 10s3-6 8-6c2 0 3.6.96 4.9 2.1M18 10s-3 6-8 6c-2 0-3.6-.96-4.9-2.1" stroke-linecap="round" />
			<path d="M3 3l14 14" stroke-linecap="round" />
		{:else}
			<path d="M2 10s3-6 8-6 8 6 8 6-3 6-8 6-8-6-8-6z" />
			<circle cx="10" cy="10" r="2.5" />
		{/if}
	</svg>
{/snippet}

<!-- E1: outline + diagonal slash on (current baseline) -->
{#snippet eyeE1(on: boolean)}{@render eyeBaseline(on)}{/snippet}

<!-- E2: filled almond eye, filled + crossed -->
{#snippet eyeE2(on: boolean)}
	<svg class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor" aria-hidden="true">
		{#if on}
			<path d="M2.5 10c1.5-2.5 4.3-5 7.5-5 1.4 0 2.7.5 3.8 1.2L4.3 16.5 3 15.2zM17.5 10c-1.5 2.5-4.3 5-7.5 5-1 0-2-.2-2.9-.6l9.7-9.7c.2.4.5.8.7 1.3z" />
		{:else}
			<path d="M10 5c-4.2 0-7 4-7.8 5C3 11 5.8 15 10 15s7-4 7.8-5C17 9 14.2 5 10 5zm0 8a3 3 0 110-6 3 3 0 010 6z" />
		{/if}
	</svg>
{/snippet}

<!-- E3: minimalist almond, dot inside; off shows dot, on hides dot+slash -->
{#snippet eyeE3(on: boolean)}
	<svg class="h-4 w-4" viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
		<path d="M2 10c2-4 5-6 8-6s6 2 8 6c-2 4-5 6-8 6s-6-2-8-6z" />
		{#if !on}
			<circle cx="10" cy="10" r="1.5" fill="currentColor" stroke="none" />
		{/if}
		{#if on}
			<path d="M4 4l12 12" />
		{/if}
	</svg>
{/snippet}

<!-- E4: bracket-style ⟨ • ⟩ / ⟨ — ⟩ (terminal mono aesthetic) -->
{#snippet eyeE4(on: boolean)}
	<svg class="h-4 w-4" viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
		<path d="M3 6 L1 10 L3 14" />
		<path d="M17 6 L19 10 L17 14" />
		{#if on}
			<path d="M6 10 L14 10" />
		{:else}
			<circle cx="10" cy="10" r="1.5" fill="currentColor" stroke="none" />
		{/if}
	</svg>
{/snippet}

<!-- Switch widgets. All use the current eye (E1) on the left. -->
{#snippet switchS1(on: boolean)}
	<span class="h-3.5 w-6 shrink-0 rounded-full border border-current p-0.5 transition-colors {on ? 'bg-brand-soft' : ''}">
		<span
			class="block h-full w-1/2 rounded-full bg-current transition-transform"
			style:transform={on ? 'translateX(100%)' : 'translateX(0)'}
		></span>
	</span>
{/snippet}

{#snippet switchS2(on: boolean)}
	<span
		class="relative inline-flex h-5 w-9 shrink-0 items-center rounded-full transition-colors {on ? 'bg-brand' : 'bg-ink-faint/25'}"
	>
		<span
			class="absolute h-4 w-4 rounded-full bg-white shadow-sm transition-transform"
			style:transform={on ? 'translateX(18px)' : 'translateX(2px)'}
		></span>
	</span>
{/snippet}

{#snippet switchS3(on: boolean)}
	<span class="relative inline-flex h-4 w-9 shrink-0 items-center">
		<span class="absolute inset-x-0 top-1/2 h-px -translate-y-1/2 bg-ink-faint/50"></span>
		<span
			class="absolute h-4 w-4 rounded-full border border-current transition-all {on ? 'bg-brand' : 'bg-surface-inverse'}"
			style:transform={on ? 'translateX(20px)' : 'translateX(0)'}
		></span>
	</span>
{/snippet}

{#snippet switchS4(on: boolean)}
	<span class="relative inline-flex h-4 w-8 shrink-0 items-center border border-current">
		<span
			class="absolute top-0 bottom-0 w-1/2 bg-current transition-all"
			style:left={on ? '50%' : '0%'}
		></span>
	</span>
{/snippet}

{#snippet row(name: string, state: boolean, click: () => void, eyeRender: any, switchRender: any)}
	<button
		type="button"
		onclick={click}
		aria-pressed={state}
		class="flex items-center gap-2.5 rounded px-3 py-1.5 text-left text-sm transition-colors {state ? 'text-ink-inverse' : 'text-ink-faint hover:bg-surface-inverse-hover/60 hover:text-ink-inverse'}"
	>
		<span class="inline-flex shrink-0">{@render eyeRender(state)}</span>
		<span class="flex-1">{name}</span>
		{@render switchRender(state)}
	</button>
{/snippet}

<div class="grid h-full grid-cols-2 gap-8 overflow-auto bg-surface-card p-8">
	<section class="flex flex-col gap-4">
		<h2 class="font-mono text-xs uppercase tracking-wide text-ink-muted">Eye design (switch held constant)</h2>

		<div class="flex flex-col gap-2">
			<span class="font-mono text-[10px] uppercase tracking-wide text-ink-faint">E1 — outline + slash (current)</span>
			<div class="flex w-[216px] bg-surface-inverse p-3 text-ink-inverse-body">
				{@render row('Discrete', eye1, () => (eye1 = !eye1), eyeE1, switchS1)}
			</div>
		</div>

		<div class="flex flex-col gap-2">
			<span class="font-mono text-[10px] uppercase tracking-wide text-ink-faint">E2 — filled almond + crossed</span>
			<div class="flex w-[216px] bg-surface-inverse p-3 text-ink-inverse-body">
				{@render row('Discrete', eye2, () => (eye2 = !eye2), eyeE2, switchS1)}
			</div>
		</div>

		<div class="flex flex-col gap-2">
			<span class="font-mono text-[10px] uppercase tracking-wide text-ink-faint">E3 — minimalist dot/slash</span>
			<div class="flex w-[216px] bg-surface-inverse p-3 text-ink-inverse-body">
				{@render row('Discrete', eye3, () => (eye3 = !eye3), eyeE3, switchS1)}
			</div>
		</div>

		<div class="flex flex-col gap-2">
			<span class="font-mono text-[10px] uppercase tracking-wide text-ink-faint">E4 — bracket terminal ⟨•⟩ / ⟨—⟩</span>
			<div class="flex w-[216px] bg-surface-inverse p-3 text-ink-inverse-body">
				{@render row('Discrete', eye4, () => (eye4 = !eye4), eyeE4, switchS1)}
			</div>
		</div>
	</section>

	<section class="flex flex-col gap-4">
		<h2 class="font-mono text-xs uppercase tracking-wide text-ink-muted">Switch design (eye held constant)</h2>

		<div class="flex flex-col gap-2">
			<span class="font-mono text-[10px] uppercase tracking-wide text-ink-faint">S1 — half-fill micro pill (current)</span>
			<div class="flex w-[216px] bg-surface-inverse p-3 text-ink-inverse-body">
				{@render row('Discrete', sw1, () => (sw1 = !sw1), eyeE1, switchS1)}
			</div>
		</div>

		<div class="flex flex-col gap-2">
			<span class="font-mono text-[10px] uppercase tracking-wide text-ink-faint">S2 — iOS classic pill</span>
			<div class="flex w-[216px] bg-surface-inverse p-3 text-ink-inverse-body">
				{@render row('Discrete', sw2, () => (sw2 = !sw2), eyeE1, switchS2)}
			</div>
		</div>

		<div class="flex flex-col gap-2">
			<span class="font-mono text-[10px] uppercase tracking-wide text-ink-faint">S3 — Material line + dot</span>
			<div class="flex w-[216px] bg-surface-inverse p-3 text-ink-inverse-body">
				{@render row('Discrete', sw3, () => (sw3 = !sw3), eyeE1, switchS3)}
			</div>
		</div>

		<div class="flex flex-col gap-2">
			<span class="font-mono text-[10px] uppercase tracking-wide text-ink-faint">S4 — square terminal block</span>
			<div class="flex w-[216px] bg-surface-inverse p-3 text-ink-inverse-body">
				{@render row('Discrete', sw4, () => (sw4 = !sw4), eyeE1, switchS4)}
			</div>
		</div>
	</section>
</div>

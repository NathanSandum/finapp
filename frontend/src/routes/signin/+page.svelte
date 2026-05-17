<script lang="ts">
	import { goto } from '$app/navigation';
	import { signIn, type UserPayload } from '$lib/api/auth';
	import { authState } from '$lib/api/state.svelte';
	import { cache } from '$lib/api/cache.svelte';
	import { buildLine } from '$lib/views/income/lib';
	import { signinTrail } from '$lib/data/signinTrail';
	import { tabs } from '$lib/tabs/registry';
	import { todayLabel } from '$lib/views/today';
	import TabIcon from '$lib/components/TabIcon.svelte';
	import type { JSend } from '$lib/tabs/types';
	import FailView from '$lib/components/FailView.svelte';
	import ErrorView from '$lib/components/ErrorView.svelte';

	const dateLabel = todayLabel();

	let email = $state('');
	let password = $state('');
	let result = $state<JSend<UserPayload> | null>(null);
	let submitting = $state(false);

	let formEl = $state<HTMLFormElement | null>(null);
	let emailEl = $state<HTMLInputElement | null>(null);
	let offset = $state(0);

	$effect(() => {
		if (!formEl || !emailEl) return;
		const measure = () => {
			const formRect = formEl!.getBoundingClientRect();
			const emailRect = emailEl!.getBoundingClientRect();
			const emailCenter = emailRect.top - formRect.top + emailRect.height / 2;
			offset = emailCenter - formRect.height / 2;
		};
		measure();
		const ro = new ResizeObserver(measure);
		ro.observe(formEl);
		return () => ro.disconnect();
	});

	const sparkline = buildLine(signinTrail, 800, 360);

	async function handleSubmit(event: SubmitEvent) {
		event.preventDefault();
		submitting = true;
		result = await signIn(email, password);
		submitting = false;
		if (result.status === 'success') {
			cache.invalidateAll();
			authState.set(result.data.user);
			goto('/');
		}
	}
</script>

<div class="grid h-screen w-screen grid-cols-[240px_1fr] bg-surface-card">
	<aside class="flex flex-col gap-0 bg-surface-inverse p-3 text-ink-inverse-body opacity-40 select-none">
		<div class="flex w-full items-center justify-between rounded px-2 py-1.5">
			<span class="flex items-center gap-2">
				<span class="flex h-6 w-6 items-center justify-center rounded bg-brand text-xs font-bold text-ink">
					F
				</span>
				<span class="text-sm font-medium text-ink-inverse">FinApp</span>
			</span>
			<svg class="h-4 w-4 text-ink-muted" viewBox="0 0 20 20" fill="currentColor" aria-hidden="true">
				<path fill-rule="evenodd" d="M5 8l5 5 5-5H5z" clip-rule="evenodd" />
			</svg>
		</div>
		<p class="mb-4 px-2 pt-0.5 font-mono text-[10px] uppercase tracking-[0.1em] text-ink-muted">
			{dateLabel}
		</p>
		<nav class="flex flex-col gap-0.5">
			{#each tabs as tab (tab.id)}
				<div class="relative flex items-center gap-2.5 rounded px-3 py-1.5 text-left text-sm text-ink-faint">
					<TabIcon id={tab.id} class="h-4 w-4 shrink-0" />
					{tab.label}
				</div>
			{/each}
		</nav>
		<div class="mt-auto flex items-center gap-2 rounded p-2">
			<div class="flex h-8 w-8 items-center justify-center rounded-full bg-gradient-to-br from-avatar-from to-avatar-to text-xs font-semibold text-ink-inverse">
				?
			</div>
			<div class="flex flex-1 flex-col leading-tight">
				<span class="text-sm text-ink-inverse-body">&nbsp;</span>
			</div>
		</div>
	</aside>
	<main class="relative overflow-hidden bg-surface">
		<svg
			class="absolute inset-0 h-full w-full"
			viewBox="0 0 800 400"
			preserveAspectRatio="none"
			aria-hidden="true"
		>
			<defs>
				<linearGradient id="signinFill" x1="0" y1="0" x2="0" y2="1">
					<stop offset="0%" style:stop-color="var(--color-positive-line)" stop-opacity="0.30" />
					<stop offset="100%" style:stop-color="var(--color-positive-line)" stop-opacity="0" />
				</linearGradient>
			</defs>
			<polygon fill="url(#signinFill)" points={sparkline.area} transform="translate(0,40)" />
			<polyline
				fill="none"
				class="stroke-positive-line"
				stroke-width="1.25"
				stroke-linecap="round"
				stroke-linejoin="round"
				points={sparkline.path}
				transform="translate(0,40)"
			/>
		</svg>

	</main>
</div>

<div class="pointer-events-none fixed inset-0 z-10 flex items-center justify-center p-6">
	<form
		bind:this={formEl}
		onsubmit={handleSubmit}
		style:transform="translateY({offset}px)"
		class="pointer-events-auto flex w-full max-w-sm flex-col gap-4 border border-border bg-surface-card p-6 shadow-2xl shadow-black/30"
	>
		<h1 class="font-display text-xl text-ink">Sign in</h1>

		<label class="flex flex-col gap-1">
			<span class="text-xs uppercase tracking-wide text-ink-muted">Email</span>
			<input
				bind:this={emailEl}
				type={import.meta.env.DEV ? 'text' : 'email'}
				bind:value={email}
				required
				autocomplete="email"
				class="border border-border bg-surface px-2 py-1.5 font-mono text-sm text-ink focus:border-brand focus:outline-none"
			/>
		</label>

		<label class="flex flex-col gap-1">
			<span class="text-xs uppercase tracking-wide text-ink-muted">Password</span>
			<input
				type="password"
				bind:value={password}
				required
				autocomplete="current-password"
				class="border border-border bg-surface px-2 py-1.5 font-mono text-sm text-ink focus:border-brand focus:outline-none"
			/>
		</label>

		{#if result !== null}
			{#if result.status === 'fail'}
				<FailView {result} />
			{:else if result.status === 'error'}
				<ErrorView {result} />
			{/if}
		{/if}

		<button
			type="submit"
			disabled={submitting}
			class="bg-surface-inverse px-4 py-2 text-sm font-medium text-ink-inverse transition-colors hover:bg-surface-inverse-hover disabled:opacity-50"
		>
			{submitting ? 'Signing in…' : 'Sign in'}
		</button>
	</form>
</div>

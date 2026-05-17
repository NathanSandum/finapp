<script lang="ts">
	import { goto } from '$app/navigation';
	import { signIn, type UserPayload } from '$lib/api/auth';
	import { authState } from '$lib/api/state.svelte';
	import { cache } from '$lib/api/cache.svelte';
	import { buildLine } from '$lib/views/income/lib';
	import { signinTrail } from '$lib/data/signinTrail';
	import type { JSend } from '$lib/tabs/types';
	import FailView from '$lib/components/FailView.svelte';
	import ErrorView from '$lib/components/ErrorView.svelte';

	let email = $state('');
	let password = $state('');
	let result = $state<JSend<UserPayload> | null>(null);
	let submitting = $state(false);

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
	<aside class="bg-surface-inverse"></aside>
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
		onsubmit={handleSubmit}
		class="pointer-events-auto flex w-full max-w-sm flex-col gap-4 border border-border bg-surface-card p-6 shadow-2xl shadow-black/30"
	>
		<h1 class="font-display text-xl text-ink">Sign in</h1>

		<label class="flex flex-col gap-1">
			<span class="text-xs uppercase tracking-wide text-ink-muted">Email</span>
			<input
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

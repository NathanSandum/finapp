<script lang="ts">
	import { goto } from '$app/navigation';
	import { signIn, type UserPayload } from '$lib/api/auth';
	import { authState } from '$lib/api/state.svelte';
	import { cache } from '$lib/api/cache.svelte';
	import type { JSend } from '$lib/tabs/types';
	import FailView from '$lib/components/FailView.svelte';
	import ErrorView from '$lib/components/ErrorView.svelte';

	let email = $state(import.meta.env.DEV ? 'dev' : '');
	let password = $state(import.meta.env.DEV ? 'dev' : '');
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

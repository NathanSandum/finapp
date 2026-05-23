<script lang="ts">
	import { onMount } from 'svelte';
	import { signIn } from '$lib/auth/session';
	import { getDiscrete, setDiscrete } from '$lib/ui/discrete.svelte';
	import type { JSend } from '$lib/tabs/types';
	import FailView from '$lib/components/FailView.svelte';
	import ErrorView from '$lib/components/ErrorView.svelte';
	import { prefersReducedMotion } from '$lib/utils/motion';
	import SolidButton from '$lib/components/SolidButton.svelte';

	let { onSignedIn }: { onSignedIn: () => void } = $props();

	let email = $state(import.meta.env.DEV ? 'dev' : '');
	let password = $state(import.meta.env.DEV ? 'dev' : '');
	let result = $state<JSend<unknown> | null>(null);
	let submitting = $state(false);
	let entered = $state(false);
	let exiting = $state(false);

	let formEl = $state<HTMLFormElement | null>(null);
	let emailEl = $state<HTMLInputElement | null>(null);
	let offset = $state(0);

	onMount(() => {
		if (prefersReducedMotion()) {
			entered = true;
			return;
		}
		requestAnimationFrame(() => requestAnimationFrame(() => (entered = true)));
	});

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
		const signInResult = await signIn(email, password);
		if (signInResult.status !== 'success') {
			result = signInResult;
			submitting = false;
			return;
		}
		submitting = false;
		exiting = true;
		onSignedIn();
	}
</script>

<form
	bind:this={formEl}
	onsubmit={handleSubmit}
	class="signin-form pointer-events-auto flex w-full max-w-sm flex-col gap-4 border border-border bg-surface-card p-6 shadow-2xl shadow-black/30"
	class:entered
	class:exiting
	style:--signin-offset="{offset}px"
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

	<label class="flex items-center gap-2 text-xs text-ink-muted">
		<input
			type="checkbox"
			checked={getDiscrete()}
			onchange={(e) => setDiscrete((e.currentTarget as HTMLInputElement).checked)}
			class="h-3.5 w-3.5 accent-brand"
		/>
		<span>Discrete mode <span class="text-ink-faint">(hide balances and totals)</span></span>
	</label>

	{#if result !== null && result.status !== 'success'}
		{#key result.status}
			<div class="result-block">
				{#if result.status === 'fail'}
					<FailView {result} />
				{:else if result.status === 'error'}
					<ErrorView {result} />
				{/if}
			</div>
		{/key}
	{/if}

	<SolidButton type="submit" disabled={submitting}>
		{submitting ? 'Signing in…' : 'Sign in'}
	</SolidButton>
</form>

<style>
	.signin-form {
		opacity: 0;
		transform: translateY(calc(var(--signin-offset, 0px) + 24px));
		transition:
			opacity 520ms cubic-bezier(0.22, 1, 0.36, 1) 120ms,
			transform 520ms cubic-bezier(0.22, 1, 0.36, 1) 120ms;
	}
	.signin-form.entered {
		opacity: 1;
		transform: translateY(var(--signin-offset, 0px));
	}
	.signin-form.exiting {
		opacity: 0;
		transform: translateY(calc(var(--signin-offset, 0px) - 16px));
		transition:
			opacity 420ms cubic-bezier(0.4, 0, 1, 1) 0ms,
			transform 420ms cubic-bezier(0.4, 0, 1, 1) 0ms;
	}
	.result-block {
		animation: result-in 280ms cubic-bezier(0.22, 1, 0.36, 1) both;
	}
	@keyframes result-in {
		from {
			opacity: 0;
			transform: translateY(8px);
		}
		to {
			opacity: 1;
			transform: translateY(0);
		}
	}
	@media (prefers-reduced-motion: reduce) {
		.signin-form,
		.signin-form.entered {
			transition: opacity 0ms, transform 0ms;
		}
		.result-block {
			animation: none;
		}
	}
</style>

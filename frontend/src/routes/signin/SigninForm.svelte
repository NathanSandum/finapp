<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { signIn, type UserPayload } from '$lib/api/auth';
	import { authState } from '$lib/api/state.svelte';
	import { cache } from '$lib/api/cache.svelte';
	import type { JSend } from '$lib/tabs/types';
	import FailView from '$lib/components/FailView.svelte';
	import ErrorView from '$lib/components/ErrorView.svelte';
	import { prefersReducedMotion } from '$lib/utils/motion';

	let email = $state(import.meta.env.DEV ? 'dev' : '');
	let password = $state(import.meta.env.DEV ? 'dev' : '');
	let result = $state<JSend<UserPayload> | null>(null);
	let submitting = $state(false);
	let entered = $state(false);
	let exiting = $state(false);

	let wrapperEl = $state<HTMLDivElement | null>(null);
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
		result = await signIn(email, password);
		submitting = false;
		if (result.status === 'success') {
			cache.invalidateAll();
			authState.set(result.data.user);
			const reduced = prefersReducedMotion();
			exiting = true;
			await new Promise((resolve) => setTimeout(resolve, reduced ? 0 : 360));
			goto('/');
		}
	}
</script>

<div
	bind:this={wrapperEl}
	class="signin-wrapper contents"
	style:--signin-offset="{offset}px"
>
	<form
		bind:this={formEl}
		onsubmit={handleSubmit}
		class="signin-form pointer-events-auto flex w-full max-w-sm flex-col gap-4 border border-border bg-surface-card p-6 shadow-2xl shadow-black/30"
		class:entered
		class:exiting
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

		<button
			type="submit"
			disabled={submitting}
			class="bg-surface-inverse px-4 py-2 text-sm font-medium text-ink-inverse transition-colors hover:bg-surface-inverse-hover disabled:opacity-50"
		>
			{submitting ? 'Signing in…' : 'Sign in'}
		</button>
	</form>
</div>

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
		transform: translateY(var(--signin-offset, 0px)) scale(0.96);
		transition:
			opacity 320ms cubic-bezier(0.4, 0, 1, 1),
			transform 360ms cubic-bezier(0.4, 0, 1, 1);
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
		.signin-form.entered,
		.signin-form.exiting {
			transition: opacity 0ms, transform 0ms;
		}
		.result-block {
			animation: none;
		}
	}
</style>

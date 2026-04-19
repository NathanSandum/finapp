<script lang="ts">
	import { goto } from '$app/navigation';
	import * as api from '$lib/api.svelte';

	let mode: 'login' | 'register' = $state('login');
	let email = $state('');
	let displayName = $state('');
	let password = $state('');
	let error = $state('');
	let loading = $state(false);

	$effect(() => {
		if (api.getToken()) goto('/');
	});

	async function submit() {
		error = '';
		loading = true;
		try {
			if (mode === 'register') {
				const data = await api.register(email, displayName, password);
				api.setToken(data.token);
			} else {
				const data = await api.login(email, password);
				api.setToken(data.token);
			}
			goto('/');
		} catch (e: unknown) {
			const err = e as { message?: string };
			error = err.message || 'Something went wrong';
		} finally {
			loading = false;
		}
	}
</script>

<div class="flex min-h-screen items-center justify-center bg-gray-950">
	<div class="w-full max-w-sm rounded-xl border border-gray-800 bg-gray-900 p-8">
		<h1 class="text-xl font-semibold text-gray-100">FinApp</h1>
		<p class="mt-1 text-sm text-gray-500">
			{mode === 'login' ? 'Sign in to your account' : 'Create an account'}
		</p>

		{#if error}
			<div class="mt-4 rounded-md border border-red-900 bg-red-950 px-3 py-2 text-sm text-red-400">
				{error}
			</div>
		{/if}

		<form
			class="mt-6 space-y-4"
			onsubmit={(e) => {
				e.preventDefault();
				submit();
			}}
		>
			<label class="block">
				<span class="text-sm text-gray-400">Email</span>
				<input
					type="email"
					bind:value={email}
					required
					class="mt-1 block w-full rounded-md border border-gray-700 bg-gray-950 px-3 py-2 text-sm text-gray-100 focus:border-blue-500 focus:outline-none"
				/>
			</label>

			{#if mode === 'register'}
				<label class="block">
					<span class="text-sm text-gray-400">Display name</span>
					<input
						type="text"
						bind:value={displayName}
						required
						class="mt-1 block w-full rounded-md border border-gray-700 bg-gray-950 px-3 py-2 text-sm text-gray-100 focus:border-blue-500 focus:outline-none"
					/>
				</label>
			{/if}

			<label class="block">
				<span class="text-sm text-gray-400">Password</span>
				<input
					type="password"
					bind:value={password}
					required
					class="mt-1 block w-full rounded-md border border-gray-700 bg-gray-950 px-3 py-2 text-sm text-gray-100 focus:border-blue-500 focus:outline-none"
				/>
			</label>

			<button
				type="submit"
				disabled={loading}
				class="w-full rounded-md bg-blue-600 px-3 py-2 text-sm font-medium text-white hover:bg-blue-700 disabled:opacity-50"
			>
				{loading ? '...' : mode === 'login' ? 'Sign in' : 'Create account'}
			</button>
		</form>

		<p class="mt-5 text-center text-sm text-gray-500">
			{#if mode === 'login'}
				No account?
				<button class="text-blue-400 hover:underline" onclick={() => (mode = 'register')}
					>Register</button
				>
			{:else}
				Have an account?
				<button class="text-blue-400 hover:underline" onclick={() => (mode = 'login')}
					>Sign in</button
				>
			{/if}
		</p>
	</div>
</div>

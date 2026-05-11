<script lang="ts">
	let {
		name,
		onEdit
	}: {
		name: string;
		onEdit: () => void;
	} = $props();
</script>

<button
	type="button"
	onclick={onEdit}
	class="flex-1 truncate text-left text-sm text-ink hover:text-ink-body"
>
	{name}
</button>

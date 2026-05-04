<script lang="ts">
	import Area from './variants/Area.svelte';
	// Alt chart style kept in repo: ./variants/Bars.svelte — swap import + tag below to use it.

	let { data: _data }: { data: null } = $props();
</script>

<Area />

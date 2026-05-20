<script lang="ts">
	let { value }: { value: string; position?: number } = $props();
</script>

<span class="title-static">{value}</span>

<style>
	.title-static {
		display: inline-block;
		line-height: 1.05;
	}
</style>

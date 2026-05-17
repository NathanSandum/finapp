<script lang="ts">
	import { todayLabel } from '$lib/views/today';

	const dateLabel = todayLabel();
</script>

<p class="mb-4 px-2 pt-0.5 font-mono text-[10px] uppercase tracking-[0.1em] text-ink-muted">
	{dateLabel}
</p>

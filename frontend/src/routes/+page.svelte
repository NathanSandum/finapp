<script lang="ts">
  import { onMount } from 'svelte';

  let status = $state('checking...');

  onMount(async () => {
    try {
      const res = await fetch('/api/health');
      const data = await res.json();
      status = data.status;
    } catch {
      status = 'backend not running';
    }
  });
</script>

<main>
  <h1>FinApp</h1>
  <p>Backend: <code>{status}</code></p>
</main>

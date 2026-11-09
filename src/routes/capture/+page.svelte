<script lang="ts">
	import DataEntryForm from '$lib/components/DataEntryForm.svelte';
</script>

<svelte:head>
	<title>CRM · Captura local</title>
	<meta
		name="description"
		content="Captura y administra clientes y propiedades de forma local y cifrada."
	/>
</svelte:head>

<div class="min-h-screen bg-slate-950">
	<DataEntryForm />
</div>

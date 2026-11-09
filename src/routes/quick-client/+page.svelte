<script lang="ts">
	import QuickClientForm from '$lib/components/QuickClientForm.svelte';
</script>

<svelte:head>
	<title>CRM · Alta rápida de lead</title>
	<meta
		name="description"
		content="Registra rápidamente un lead y guárdalo de forma cifrada en este dispositivo."
	/>
</svelte:head>

<main class="min-h-screen bg-slate-950 px-4 py-8 sm:px-6">
	<div class="mx-auto mb-4 max-w-2xl">
		<a
			href="/capture"
			class="text-sm text-sky-200 underline decoration-sky-200/40 underline-offset-4 hover:text-white focus-visible:outline-2 focus-visible:outline-sky-300"
		>
			Ir a captura completa de clientes y propiedades
		</a>
	</div>
	<QuickClientForm />
</main>

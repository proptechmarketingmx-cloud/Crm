<script lang="ts">
	import { useMatchingQuery } from '$lib/queries/matching';
	import type { MatchRequest } from '$lib/services/matching';
	import ClientNetworkGraph from '$lib/components/ClientNetworkGraph.svelte';
	import EGAHeatmap from '$lib/components/EGAHeatmap.svelte';
	import MatchingResults from '$lib/components/MatchingResults.svelte';

	let { request }: { request: MatchRequest } = $props();
	let selectedPropertyId = $state<string | null>(null);
	const matching = useMatchingQuery(() => request);
	const response = $derived($matching.data);
	const matches = $derived(response?.value.matches ?? []);

	$effect(() => {
		const firstMatch = matches[0]?.propertyId;
		if (firstMatch && !matches.some((match) => match.propertyId === selectedPropertyId)) {
			selectedPropertyId = firstMatch;
		}
		if (matches.length === 0) selectedPropertyId = null;
	});
</script>

<main class="mx-auto grid w-full max-w-[1600px] gap-4 p-4 text-slate-100 sm:gap-6 sm:p-6">
	<header class="flex flex-wrap items-start justify-between gap-4">
		<div>
			<p class="text-xs font-semibold uppercase tracking-[0.18em] text-sky-300">CRM local-first</p>
			<h1 class="mt-1 text-2xl font-bold tracking-tight text-white sm:text-3xl">
				Matching predictivo
			</h1>
			<p class="mt-2 max-w-2xl text-sm leading-6 text-slate-400">
				Recomendaciones explicables que se calculan y permanecen en este dispositivo.
			</p>
		</div>
		<div class="flex flex-wrap items-center gap-2">
			<span class="inline-flex items-center gap-2 rounded-full border border-emerald-300/20 bg-emerald-300/10 px-3 py-1.5 text-xs text-emerald-100">
				<span class="h-2 w-2 rounded-full bg-emerald-300" aria-hidden="true"></span>
				Procesamiento local
			</span>
			{#if response?.source === 'cache'}
				<span class="rounded-full border border-amber-300/20 bg-amber-300/10 px-3 py-1.5 text-xs text-amber-100" role="status">
					Modo sin conexión · copia local
				</span>
			{:else}
				<span class="rounded-full border border-white/10 bg-white/5 px-3 py-1.5 text-xs text-slate-300" role="status">
					Datos del dispositivo
				</span>
			{/if}
		</div>
	</header>

	{#if response?.cacheWarning}
		<p class="rounded-xl border border-amber-300/20 bg-amber-300/5 px-4 py-3 text-sm text-amber-100" role="status">
			{response.cacheWarning}
		</p>
	{/if}

	{#if $matching.isPending}
		<div class="grid min-h-48 place-items-center rounded-2xl border border-white/10 bg-slate-900" aria-live="polite">
			<p class="text-sm text-slate-300">Consultando datos locales de matching…</p>
		</div>
	{:else if $matching.isError}
		<div class="rounded-2xl border border-rose-300/20 bg-rose-300/5 p-5" role="alert">
			<h2 class="font-semibold text-rose-100">No se pudo cargar el matching local</h2>
			<p class="mt-1 text-sm text-rose-100/80">
				No hay una copia guardada disponible. La interfaz no intentará consultar servicios externos.
			</p>
			<button
				class="mt-4 rounded-lg border border-rose-200/30 px-3 py-2 text-sm text-white hover:bg-rose-200/10 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-rose-200"
				onclick={() => $matching.refetch()}
			>
				Reintentar comando local
			</button>
		</div>
	{:else}
		<div class="grid min-w-0 gap-4 xl:grid-cols-[minmax(0,1.15fr)_minmax(340px,0.85fr)]">
			<MatchingResults
				{matches}
				{selectedPropertyId}
				onSelect={(propertyId) => (selectedPropertyId = propertyId)}
			/>
			<div class="grid min-w-0 content-start gap-4">
				<ClientNetworkGraph clientId={request.clientId} />
				<EGAHeatmap />
			</div>
		</div>
	{/if}

	<footer class="border-t border-white/10 pt-4 text-xs leading-5 text-slate-500">
		Las puntuaciones y sus componentes provienen de comandos Tauri locales. Las copias offline se guardan
		en IndexedDB de este perfil de usuario.
	</footer>
</main>

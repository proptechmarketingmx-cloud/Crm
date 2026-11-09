<script lang="ts">
	import cytoscape, { type Core } from 'cytoscape';
	import { onMount } from 'svelte';
	import { createClientNetworkQuery } from '$lib/queries/matching';

	let { clientId }: { clientId: string } = $props();
	let graphElement: HTMLDivElement;
	let graph = $state<Core | undefined>();
	const network = createClientNetworkQuery(() => clientId);
	const networkData = $derived($network.data?.value);

	onMount(() => {
		graph = cytoscape({
			container: graphElement,
			elements: [],
			wheelSensitivity: 0.25,
			minZoom: 0.25,
			maxZoom: 2,
			style: [
				{
					selector: 'node',
					style: {
						label: 'data(label)',
						'font-size': 10,
						color: '#e2e8f0',
						'text-wrap': 'wrap',
						'text-max-width': 80,
						'background-color': '#38bdf8',
						width: 22,
						height: 22,
						'border-width': 2,
						'border-color': '#0f172a'
					}
				},
				{
					selector: 'node[activity = "purchase"]',
					style: { 'background-color': '#34d399', width: 30, height: 30 }
				},
				{
					selector: 'node[activity = "visit"]',
					style: { 'background-color': '#fbbf24' }
				},
				{
					selector: 'node[activity = "search"]',
					style: { 'background-color': '#a78bfa' }
				},
				{
					selector: 'edge',
					style: {
						width: 'mapData(weight, 0, 1, 1, 5)',
						'line-color': '#64748b',
						'curve-style': 'bezier',
						opacity: 0.8
					}
				},
				{
					selector: ':selected',
					style: { 'border-color': '#f8fafc', 'border-width': 4 }
				}
			],
			layout: { name: 'circle', animate: false, fit: true, padding: 24 }
		});

		return () => {
			graph?.destroy();
			graph = undefined;
		};
	});

	$effect(() => {
		const data = networkData;
		if (!graph || !data) return;

		graph.batch(() => {
			graph.elements().remove();
			graph.add([
				...data.nodes.slice(0, 501).map((node) => ({
					group: 'nodes' as const,
					data: { id: node.id, label: node.label, activity: node.activity }
				})),
				...data.edges.slice(0, 500).map((edge, index) => ({
					group: 'edges' as const,
					data: {
						id: `edge-${index}`,
						source: edge.source,
						target: edge.target,
						weight: edge.weight,
						signals: edge.signals.join(', ')
					}
				}))
			]);
		});
		graph.layout({ name: 'circle', animate: false, fit: true, padding: 24 }).run();
	});
</script>

<section class="min-w-0 rounded-2xl border border-white/10 bg-slate-900 p-4 sm:p-5" aria-labelledby="network-title">
	<div class="mb-3">
		<h2 id="network-title" class="text-lg font-semibold text-white">Red CNA</h2>
		<p class="text-sm text-slate-400">Relaciones e interacciones del cliente seleccionado</p>
	</div>

	{#if $network.isPending}
		<p class="grid h-72 place-items-center text-sm text-slate-400" aria-live="polite">
			Cargando grafo local…
		</p>
	{:else if $network.isError}
		<div class="grid h-72 place-items-center rounded-xl bg-slate-950/60 p-5 text-center" role="alert">
			<p class="text-sm text-slate-300">No hay red CNA local disponible para este cliente.</p>
			<button
				class="mt-3 rounded-lg border border-white/20 px-3 py-2 text-sm text-white hover:bg-white/10 focus-visible:outline-2 focus-visible:outline-sky-300"
				onclick={() => $network.refetch()}
			>
				Reintentar
			</button>
		</div>
	{:else}
		<div
			bind:this={graphElement}
			class="h-72 w-full rounded-xl bg-slate-950/60"
			role="img"
			aria-label="Grafo de relaciones del cliente; las compras se muestran en verde, visitas en ámbar y búsquedas en violeta"
		></div>
		<div class="mt-3 flex flex-wrap gap-x-4 gap-y-2 text-xs text-slate-400" aria-label="Leyenda del grafo">
			<span><span class="text-emerald-300" aria-hidden="true">●</span> Compra</span>
			<span><span class="text-amber-300" aria-hidden="true">●</span> Visita</span>
			<span><span class="text-violet-300" aria-hidden="true">●</span> Búsqueda</span>
			<span><span class="text-sky-300" aria-hidden="true">●</span> Sin actividad</span>
		</div>
		{#if networkData}
			<ul class="sr-only" aria-label="Relaciones en formato de texto">
				{#each networkData.edges.slice(0, 500) as edge, index (`${edge.source}-${edge.target}-${index}`)}
					<li>
						Relación con {networkData.nodes.find((node) => node.id === edge.target)?.label ??
							'cliente relacionado'}, fuerza {edge.weight.toFixed(2)}.
						Señales: {edge.signals.join(', ') || 'sin detalle disponible'}.
					</li>
				{/each}
			</ul>
		{/if}
	{/if}

	{#if $network.data?.source === 'cache'}
		<p class="mt-3 text-xs text-amber-200" role="status">Mostrando grafo guardado en este dispositivo.</p>
	{/if}
	{#if $network.data?.cacheWarning}
		<p class="mt-2 text-xs text-amber-200" role="status">{$network.data.cacheWarning}</p>
	{/if}
</section>

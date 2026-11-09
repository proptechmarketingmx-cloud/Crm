<script lang="ts">
	import maplibregl, { type Map as MapLibreMap } from 'maplibre-gl';
	import { onMount } from 'svelte';
	import { createEgaHeatmapQuery } from '$lib/queries/matching';
	import type { EGAHeatmapLayer } from '$lib/services/matching';
	import 'maplibre-gl/dist/maplibre-gl.css';

	const ega = createEgaHeatmapQuery();
	const heatmap = $derived($ega.data?.value);
	let mapElement: HTMLDivElement;
	let map: MapLibreMap | undefined;
	let styleReady = $state(false);

	onMount(() => {
		map = new maplibregl.Map({
			container: mapElement,
			style: {
				version: 8,
				sources: {},
				layers: [{ id: 'background', type: 'background', paint: { 'background-color': '#0f172a' } }]
			},
			center: [-99.1332, 19.4326],
			zoom: 8,
			attributionControl: false,
			interactive: true,
			renderWorldCopies: false
		});
		map.addControl(new maplibregl.NavigationControl({ showCompass: false }), 'top-right');
		map.on('load', () => {
			if (!map) return;
			map.addSource('ega-cells', {
				type: 'geojson',
				data: emptyFeatureCollection()
			});
			map.addLayer({
				id: 'ega-cells-fill',
				type: 'fill',
				source: 'ega-cells',
				paint: {
					'fill-color': [
						'interpolate',
						['linear'],
						['coalesce', ['get', 'score'], 0],
						0,
						'#1e3a8a',
						0.35,
						'#06b6d4',
						0.65,
						'#facc15',
						1,
						'#f43f5e'
					],
					'fill-opacity': 0.68,
					'fill-outline-color': '#e2e8f0'
				}
			});
			styleReady = true;
		});

		return () => {
			map?.remove();
			map = undefined;
		};
	});

	$effect(() => {
		if (!styleReady || !heatmap || !map) return;
		updateHeatmap(map, heatmap);
	});

	function emptyFeatureCollection(): EGAHeatmapLayer {
		return { type: 'FeatureCollection', features: [] };
	}

	function updateHeatmap(instance: MapLibreMap, layer: EGAHeatmapLayer): void {
		const source = instance.getSource('ega-cells');
		if (source?.type === 'geojson') source.setData(layer);
		let minLongitude = Infinity;
		let maxLongitude = -Infinity;
		let minLatitude = Infinity;
		let maxLatitude = -Infinity;
		for (const feature of layer.features) {
			for (const ring of feature.geometry.coordinates) {
				for (const [longitude, latitude] of ring) {
					minLongitude = Math.min(minLongitude, longitude);
					maxLongitude = Math.max(maxLongitude, longitude);
					minLatitude = Math.min(minLatitude, latitude);
					maxLatitude = Math.max(maxLatitude, latitude);
				}
			}
		}
		if (!Number.isFinite(minLongitude)) return;
		instance.fitBounds(
			[
				[minLongitude, minLatitude],
				[maxLongitude, maxLatitude]
			],
			{ padding: 36, maxZoom: 12, duration: 0 }
		);
	}
</script>

<section class="min-w-0 rounded-2xl border border-white/10 bg-slate-900 p-4 sm:p-5" aria-labelledby="ega-title">
	<div class="mb-3 flex flex-wrap items-start justify-between gap-3">
		<div>
			<h2 id="ega-title" class="text-lg font-semibold text-white">Gravedad económica (EGA)</h2>
			<p class="text-sm text-slate-400">Celdas locales coloreadas por score de zona</p>
		</div>
		<div class="flex items-center gap-2 text-xs text-slate-400" aria-label="Leyenda de score EGA">
			<span>Menor</span><span class="h-2 w-20 rounded bg-gradient-to-r from-blue-800 via-cyan-500 via-yellow-400 to-rose-500"></span><span>Mayor</span>
		</div>
	</div>

	{#if $ega.isPending}
		<p class="grid h-72 place-items-center text-sm text-slate-400" aria-live="polite">
			Cargando capa EGA local…
		</p>
	{:else if $ega.isError}
		<div class="grid h-72 place-items-center rounded-xl bg-slate-950/60 p-5 text-center" role="alert">
			<p class="text-sm text-slate-300">No hay capa EGA local disponible.</p>
			<button
				class="mt-3 rounded-lg border border-white/20 px-3 py-2 text-sm text-white hover:bg-white/10 focus-visible:outline-2 focus-visible:outline-sky-300"
				onclick={() => $ega.refetch()}
			>
				Reintentar
			</button>
		</div>
	{:else}
		<div
			bind:this={mapElement}
			class="h-72 w-full overflow-hidden rounded-xl bg-slate-950/60"
			role="application"
			aria-label="Mapa interactivo local de intensidad EGA. Usa las teclas de flecha para desplazarte y más o menos para acercar o alejar."
		></div>
		{#if heatmap && heatmap.features.length === 0}
			<p class="mt-2 text-xs text-slate-400" role="status">No hay celdas EGA calculadas todavía.</p>
		{/if}
	{/if}

	{#if $ega.data?.source === 'cache'}
		<p class="mt-3 text-xs text-amber-200" role="status">Mostrando mapa guardado en este dispositivo.</p>
	{/if}
	{#if $ega.data?.cacheWarning}
		<p class="mt-2 text-xs text-amber-200" role="status">{$ega.data.cacheWarning}</p>
	{/if}
</section>

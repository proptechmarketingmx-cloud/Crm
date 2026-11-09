<script lang="ts">
	import type { PropertyMatch } from '$lib/services/matching';

	let {
		matches,
		selectedPropertyId,
		onSelect
	}: {
		matches: PropertyMatch[];
		selectedPropertyId: string | null;
		onSelect: (propertyId: string) => void;
	} = $props();

	const explainedMatches = $derived.by(() =>
		matches.map((match) => ({
			...match,
			signals: [
				{
					label: 'Coincidencia de embedding',
					value: match.breakdown.contentSimilarity,
					className: 'border-sky-300/30 bg-sky-400/10 text-sky-100'
				},
				{
					label: 'Cascada de red',
					value: match.breakdown.cascadeScore,
					className: 'border-violet-300/30 bg-violet-400/10 text-violet-100'
				},
				{
					label: 'Gravedad económica',
					value: match.breakdown.egaScore,
					className: 'border-amber-300/30 bg-amber-400/10 text-amber-100'
				}
			]
		}))
	);
</script>

<section class="min-w-0 rounded-2xl border border-white/10 bg-slate-900 p-4 sm:p-5" aria-labelledby="matches-title">
	<div class="mb-4 flex items-center justify-between gap-3">
		<div>
			<h2 id="matches-title" class="text-lg font-semibold text-white">Propiedades recomendadas</h2>
			<p class="text-sm text-slate-400">Ponderaciones calculadas localmente</p>
		</div>
		<span class="rounded-full bg-white/5 px-3 py-1 text-xs text-slate-300">
			{matches.length} resultados
		</span>
	</div>

	{#if matches.length === 0}
		<p class="rounded-xl border border-dashed border-white/15 p-5 text-sm text-slate-400">
			No hay propiedades para mostrar con los filtros actuales.
		</p>
	{:else}
		<ul class="grid gap-3">
			{#each explainedMatches as match (match.propertyId)}
				<li>
					<button
						type="button"
						class="w-full rounded-xl border p-4 text-left transition-colors focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-sky-300 {selectedPropertyId ===
						match.propertyId
							? 'border-sky-300/60 bg-sky-400/10'
							: 'border-white/10 bg-slate-950/60 hover:border-white/25'}"
						aria-pressed={selectedPropertyId === match.propertyId}
						onclick={() => onSelect(match.propertyId)}
					>
						<div class="flex flex-wrap items-start justify-between gap-3">
							<div>
								<h3 class="font-medium text-white">Propiedad {match.propertyId.slice(0, 8)}</h3>
								<p class="mt-1 text-xs text-slate-400">
									A {match.distanceKm.toFixed(1)} km · score {match.totalScore.toFixed(3)}
								</p>
							</div>
							<span class="rounded-lg bg-white/10 px-2.5 py-1 text-sm font-semibold text-white">
								{match.totalScore.toFixed(3)}
							</span>
						</div>

						<ul class="mt-3 flex flex-wrap gap-2" aria-label="Desglose explicable de la puntuación">
							{#each match.signals as signal (signal.label)}
								<li>
									<span class="inline-flex items-center gap-1.5 rounded-full border px-2.5 py-1 text-xs {signal.className}">
										<span aria-hidden="true">●</span>
										{signal.label}: {signal.value.toFixed(3)}
									</span>
								</li>
							{/each}
						</ul>

						<p class="mt-3 text-sm leading-6 text-slate-300">{match.explanation}</p>
						{#if match.evidence}
							<p class="mt-2 break-all text-xs text-slate-500">
								Evidencia local: {match.evidence.evidenceType}, fuente {match.evidence.sourceHash},
								consentimiento {match.evidence.consentId}
							</p>
						{/if}
					</button>
				</li>
			{/each}
		</ul>
	{/if}
</section>

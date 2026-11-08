<script lang="ts">
	import { createPredictionQuery } from '$lib/queries/prediction';
	import type { PredictionQueryInput } from '$lib/services/prediction';

	let { input }: { input: PredictionQueryInput } = $props();
	const prediction = createPredictionQuery(() => input);
</script>

{#if $prediction.isPending}
	<p aria-live="polite">Calculando predicción local…</p>
{:else if $prediction.isError}
	<p role="alert">No fue posible calcular la predicción. Verifica los datos locales.</p>
{:else if $prediction.data}
	<section aria-label="Resultado de recomendación">
		<p>Probabilidad estimada: {($prediction.data.purchaseProbability * 100).toFixed(1)}%</p>
		<p>Score de compatibilidad: {$prediction.data.matchScore.toFixed(3)}</p>
		<dl>
			<dt>Similitud de contenido</dt>
			<dd>{$prediction.data.cosineSimilarity.toFixed(3)}</dd>
			<dt>Contribución de red</dt>
			<dd>{$prediction.data.networkContribution.toFixed(3)}</dd>
			<dt>Contribución geográfica</dt>
			<dd>{$prediction.data.geographicContribution.toFixed(3)}</dd>
			<dt>Contribución temporal</dt>
			<dd>{$prediction.data.temporalContribution.toFixed(3)}</dd>
		</dl>
	</section>
{/if}

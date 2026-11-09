<script lang="ts">
	import type { DraftSummary } from '$lib/services/data-entry';

	let {
		drafts,
		draftsError,
		onRestore
	}: {
		drafts: DraftSummary[];
		draftsError: string;
		onRestore: (draft: DraftSummary) => void;
	} = $props();
</script>

<section class="grid gap-3 rounded-2xl border border-white/10 bg-slate-900 p-4" aria-labelledby="drafts-title">
	<div>
		<h2 id="drafts-title" class="font-semibold text-white">Borradores recuperables</h2>
		<p class="mt-1 text-xs text-slate-400">Guardados cifrados en este dispositivo.</p>
	</div>
	{#if draftsError}
		<p class="text-xs text-amber-200" role="status">{draftsError}</p>
	{:else if drafts.length === 0}
		<p class="text-xs text-slate-500">No hay borradores pendientes.</p>
	{:else}
		<ul class="grid max-h-80 gap-2 overflow-auto">
			{#each drafts as draft (draft.id)}
				<li>
					<button type="button" class="w-full rounded-lg border border-white/10 bg-slate-950/50 p-3 text-left hover:border-white/25 focus-visible:outline-2 focus-visible:outline-sky-300" onclick={() => onRestore(draft)}>
						<span class="block text-sm text-white">{draft.label || (draft.kind === 'client' ? 'Cliente sin nombre' : 'Propiedad sin tipo')}</span>
						<span class="mt-1 block text-xs text-slate-500">{draft.kind === 'client' ? 'Cliente' : 'Propiedad'} · {new Date(draft.updatedAt).toLocaleString()}</span>
					</button>
				</li>
			{/each}
		</ul>
	{/if}
</section>

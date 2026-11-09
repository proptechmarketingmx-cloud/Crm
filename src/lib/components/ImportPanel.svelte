<script lang="ts">
	import type { ClientFormState } from '$lib/services/data-entry';

	let {
		importQueue,
		importing,
		importError,
		importNotice,
		onFile,
		onSave
	}: {
		importQueue: ClientFormState[];
		importing: boolean;
		importError: string;
		importNotice: string;
		onFile: (event: Event) => void;
		onSave: () => void;
	} = $props();
</script>

<section class="grid gap-3 rounded-2xl border border-white/10 bg-slate-900 p-4" aria-labelledby="import-title">
	<div>
		<h2 id="import-title" class="font-semibold text-white">Importación masiva</h2>
		<p class="mt-1 text-xs leading-5 text-slate-400">CSV con encabezados o vCard (.vcf). El archivo se lee solo en memoria local.</p>
	</div>
	<label class="grid gap-2 text-sm text-slate-300">
		Seleccionar archivo
		<input
			type="file"
			accept=".csv,text/csv,.vcf,text/vcard"
			onchange={onFile}
			class="block w-full text-xs text-slate-400 file:mr-3 file:rounded-lg file:border-0 file:bg-slate-700 file:px-3 file:py-2 file:text-sm file:text-white hover:file:bg-slate-600"
		/>
	</label>
	{#if importError}<p class="text-xs text-rose-200" role="alert">{importError}</p>{/if}
	{#if importNotice}<p class="text-xs text-emerald-200" role="status">{importNotice}</p>{/if}
	{#if importQueue.length}
		<button type="button" disabled={importing} onclick={onSave} class="rounded-lg border border-sky-200/25 px-3 py-2 text-sm text-sky-100 hover:bg-sky-200/10 disabled:opacity-40 focus-visible:outline-2 focus-visible:outline-sky-200">
			{importing ? 'Guardando…' : `Guardar ${importQueue.length} como borradores cifrados`}
		</button>
	{/if}
</section>

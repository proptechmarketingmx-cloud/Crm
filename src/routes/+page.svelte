<script lang="ts">
	import MatchingDashboard from '$lib/components/MatchingDashboard.svelte';
	import type { MatchRequest } from '$lib/services/matching';

	let clientId = $state('');
	let activeRequest = $state<MatchRequest | null>(null);

	function loadDashboard(event: SubmitEvent): void {
		event.preventDefault();
		const normalizedId = clientId.trim();
		if (!normalizedId) return;
		activeRequest = { clientId: normalizedId, topK: 10, maxDistanceKm: 15 };
	}
</script>

<svelte:head>
	<title>CRM · Matching local</title>
	<meta
		name="description"
		content="Recomendaciones inmobiliarias explicables calculadas localmente."
	/>
</svelte:head>

<div class="min-h-screen bg-slate-950">
	<nav class="mx-auto flex max-w-6xl justify-end gap-2 px-6 pt-5" aria-label="Navegación principal">
		<a
			href="/quick-client"
			class="rounded-lg border border-sky-300/30 px-3 py-2 text-sm font-medium text-sky-100 hover:bg-sky-300/10 focus-visible:outline-2 focus-visible:outline-sky-300"
		>
			Alta rápida
		</a>
		<a
			href="/capture"
			class="rounded-lg border border-white/15 px-3 py-2 text-sm font-medium text-slate-200 hover:bg-white/5 focus-visible:outline-2 focus-visible:outline-sky-300"
		>
			Capturar clientes y propiedades
		</a>
	</nav>
	{#if activeRequest}
		<MatchingDashboard request={activeRequest} />
	{:else}
		<main class="mx-auto grid min-h-screen max-w-xl content-center gap-6 p-6 text-slate-100">
			<div>
				<p class="text-xs font-semibold uppercase tracking-[0.18em] text-sky-300">
					CRM local-first
				</p>
				<h1 class="mt-2 text-3xl font-bold text-white">Matching predictivo</h1>
				<p class="mt-3 text-sm leading-6 text-slate-400">
					Selecciona un cliente por su identificador local. La consulta se procesa mediante
					comandos Tauri y no contacta servicios externos.
				</p>
			</div>

			<form
				class="grid gap-4 rounded-2xl border border-white/10 bg-slate-900 p-5"
				onsubmit={loadDashboard}
			>
				<label for="client-id" class="grid gap-2 text-sm font-medium text-slate-200">
					ID local del cliente
					<input
						id="client-id"
						bind:value={clientId}
						autocomplete="off"
						spellcheck="false"
						required
						pattern="[0-9a-fA-F-]{36}"
						title="Ingresa un UUID de 36 caracteres."
						class="rounded-lg border border-white/15 bg-slate-950 px-3 py-2.5 font-mono text-sm text-white outline-none focus-visible:border-sky-300 focus-visible:ring-2 focus-visible:ring-sky-300/30"
					/>
				</label>
				<button
					type="submit"
					class="rounded-lg bg-sky-400 px-4 py-2.5 text-sm font-semibold text-slate-950 hover:bg-sky-300 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-sky-200"
				>
					Abrir dashboard local
				</button>
			</form>
		</main>
	{/if}
</div>

<script lang="ts">
	import {
		saveClientLocally,
		validateQuickClient,
		type ClientField,
		type ClientPayload,
		type FormErrors
	} from '$lib/services/quick-client';

	type QuickClientDraft = Omit<ClientPayload, 'id' | 'presupuestoMaximo'> & {
		presupuestoMaximo: string;
	};
	type SaveJob = { payload: ClientPayload; status: 'saving' | 'error' };

	const EMPTY_FORM: QuickClientDraft = {
		nombre: '',
		apellidoPaterno: '',
		telefono: '',
		presupuestoMaximo: '',
		zonaInteres: '',
		consentimientoEnriquecimiento: false
	};

	const FIELD_ORDER: ClientField[] = [
		'nombre',
		'apellidoPaterno',
		'telefono',
		'presupuestoMaximo',
		'zonaInteres'
	];

	let form = $state<QuickClientDraft>({ ...EMPTY_FORM });
	let touched = $state<Partial<Record<ClientField, boolean>>>({});
	let attempted = $state(false);
	let jobs = $state<SaveJob[]>([]);
	let notice = $state('');
	let noticeKind = $state<'success' | 'pending' | 'error'>('pending');
	let formElement: HTMLFormElement | undefined;

	const errors = $derived.by((): FormErrors =>
		validateQuickClient({
			...form,
			presupuestoMaximo: Number(form.presupuestoMaximo)
		})
	);
	const hasErrors = $derived(Object.keys(errors).length > 0);

	function fieldError(field: ClientField): string | undefined {
		return (attempted || touched[field]) ? errors[field] : undefined;
	}

	async function submit(event: SubmitEvent): Promise<void> {
		event.preventDefault();
		attempted = true;
		if (hasErrors) {
			const firstInvalid = FIELD_ORDER.find((field) => errors[field]);
			if (firstInvalid) {
				document.getElementById(`quick-client-${firstInvalid}`)?.focus();
			}
			return;
		}

		const payload: ClientPayload = {
			id: crypto.randomUUID(),
			nombre: form.nombre.trim(),
			apellidoPaterno: form.apellidoPaterno.trim(),
			telefono: form.telefono.trim(),
			presupuestoMaximo: Number(form.presupuestoMaximo),
			zonaInteres: form.zonaInteres.trim(),
			consentimientoEnriquecimiento: form.consentimientoEnriquecimiento
		};

		form = { ...EMPTY_FORM };
		touched = {};
		attempted = false;
		notice = 'Guardando localmente…';
		noticeKind = 'pending';
		void persist(payload);
		requestAnimationFrame(() => formElement?.querySelector<HTMLInputElement>('#quick-client-nombre')?.focus());
	}

	async function persist(payload: ClientPayload): Promise<void> {
		const existingJob = jobs.find((job) => job.payload.id === payload.id);
		if (existingJob?.status === 'saving') return;
		jobs = jobs.some((job) => job.payload.id === payload.id)
			? jobs.map((job) =>
					job.payload.id === payload.id ? { ...job, status: 'saving' } : job
				)
			: [...jobs, { payload, status: 'saving' }];
		try {
			await saveClientLocally(payload);
			jobs = jobs.filter((job) => job.payload.id !== payload.id);
			notice = '✓ Lead guardado localmente como borrador.';
			noticeKind = 'success';
		} catch {
			jobs = jobs.map((job) =>
				job.payload.id === payload.id ? { ...job, status: 'error' } : job
			);
			notice = 'No se pudo guardar este lead. Los datos siguen disponibles para reintentar.';
			noticeKind = 'error';
		}
	}

	function markTouched(field: ClientField): void {
		touched[field] = true;
	}
</script>

<section class="mx-auto w-full max-w-2xl rounded-2xl border border-white/10 bg-slate-900 p-5 text-slate-100 shadow-xl shadow-black/20 sm:p-7">
	<header class="mb-6">
		<p class="text-xs font-semibold uppercase tracking-[0.18em] text-sky-300">Alta rápida · Local</p>
		<h1 class="mt-1 text-2xl font-bold text-white">Nuevo lead</h1>
		<p class="mt-2 text-sm leading-6 text-slate-400">
			Se guarda en este dispositivo. El consentimiento de enriquecimiento es opcional y no inicia búsquedas.
		</p>
	</header>

	{#if notice}
		<p
			class="mb-5 rounded-lg border px-3 py-2.5 text-sm {noticeKind === 'success'
				? 'border-emerald-300/25 bg-emerald-300/5 text-emerald-100'
				: noticeKind === 'error'
					? 'border-rose-300/25 bg-rose-300/5 text-rose-100'
					: 'border-sky-300/25 bg-sky-300/5 text-sky-100'}"
			role={noticeKind === 'error' ? 'alert' : 'status'}
			aria-live="polite"
		>
			{notice}
		</p>
	{/if}

	{#if jobs.some((job) => job.status === 'error')}
		<ul class="mb-5 grid gap-2" aria-label="Guardados pendientes de reintento">
			{#each jobs.filter((job) => job.status === 'error') as job (job.payload.id)}
				<li class="flex flex-wrap items-center justify-between gap-3 rounded-lg border border-rose-300/20 bg-rose-300/5 px-3 py-2">
					<span class="text-sm text-rose-100">
						{job.payload.nombre} {job.payload.apellidoPaterno} no se guardó.
					</span>
					<button
						type="button"
						class="rounded-md border border-rose-200/30 px-3 py-1.5 text-sm font-medium text-rose-50 hover:bg-rose-200/10 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-rose-200"
						onclick={() => void persist(job.payload)}
					>
						Reintentar
					</button>
				</li>
			{/each}
		</ul>
	{/if}

	<form
		bind:this={formElement}
		class="grid gap-4"
		novalidate
		onsubmit={submit}
	>
		<div class="grid gap-4 sm:grid-cols-2">
			<label class="grid gap-1.5 text-sm font-medium text-slate-200">
				Nombre
				<input
					id="quick-client-nombre"
					bind:value={form.nombre}
					onblur={() => markTouched('nombre')}
					autocomplete="given-name"
					required
					maxlength="120"
					aria-required="true"
					aria-invalid={Boolean(fieldError('nombre'))}
					aria-describedby={fieldError('nombre') ? 'quick-client-nombre-error' : undefined}
					class="rounded-lg border border-white/15 bg-slate-950 px-3 py-2.5 text-white outline-none transition focus-visible:border-sky-300 focus-visible:ring-2 focus-visible:ring-sky-300/30 aria-[invalid=true]:border-rose-300"
				/>
				{#if fieldError('nombre')}
					<span id="quick-client-nombre-error" class="text-xs text-rose-200">{fieldError('nombre')}</span>
				{/if}
			</label>

			<label class="grid gap-1.5 text-sm font-medium text-slate-200">
				Apellido
				<input
					id="quick-client-apellidoPaterno"
					bind:value={form.apellidoPaterno}
					onblur={() => markTouched('apellidoPaterno')}
					autocomplete="family-name"
					required
					maxlength="120"
					aria-required="true"
					aria-invalid={Boolean(fieldError('apellidoPaterno'))}
					aria-describedby={fieldError('apellidoPaterno') ? 'quick-client-apellido-error' : undefined}
					class="rounded-lg border border-white/15 bg-slate-950 px-3 py-2.5 text-white outline-none transition focus-visible:border-sky-300 focus-visible:ring-2 focus-visible:ring-sky-300/30 aria-[invalid=true]:border-rose-300"
				/>
				{#if fieldError('apellidoPaterno')}
					<span id="quick-client-apellido-error" class="text-xs text-rose-200">{fieldError('apellidoPaterno')}</span>
				{/if}
			</label>
		</div>

		<label class="grid gap-1.5 text-sm font-medium text-slate-200">
			Teléfono / WhatsApp
			<input
				id="quick-client-telefono"
				bind:value={form.telefono}
				onblur={() => markTouched('telefono')}
				type="tel"
				autocomplete="tel"
				inputmode="tel"
				required
				maxlength="20"
				aria-required="true"
				aria-invalid={Boolean(fieldError('telefono'))}
				aria-describedby={fieldError('telefono') ? 'quick-client-telefono-error' : 'quick-client-telefono-help'}
				placeholder="+52 55 1234 5678"
				class="rounded-lg border border-white/15 bg-slate-950 px-3 py-2.5 text-white outline-none transition focus-visible:border-sky-300 focus-visible:ring-2 focus-visible:ring-sky-300/30 aria-[invalid=true]:border-rose-300"
			/>
			{#if fieldError('telefono')}
				<span id="quick-client-telefono-error" class="text-xs text-rose-200">{fieldError('telefono')}</span>
			{:else}
				<span id="quick-client-telefono-help" class="text-xs text-slate-500">Incluye lada si aplica.</span>
			{/if}
		</label>

		<div class="grid gap-4 sm:grid-cols-2">
			<label class="grid gap-1.5 text-sm font-medium text-slate-200">
				Presupuesto máximo (MXN)
				<input
					id="quick-client-presupuestoMaximo"
					bind:value={form.presupuestoMaximo}
					onblur={() => markTouched('presupuestoMaximo')}
					type="text"
					inputmode="decimal"
					autocomplete="off"
					required
					maxlength="32"
					aria-required="true"
					aria-invalid={Boolean(fieldError('presupuestoMaximo'))}
					aria-describedby={fieldError('presupuestoMaximo') ? 'quick-client-presupuesto-error' : undefined}
					placeholder="Ej. 2500000"
					class="rounded-lg border border-white/15 bg-slate-950 px-3 py-2.5 text-white outline-none transition focus-visible:border-sky-300 focus-visible:ring-2 focus-visible:ring-sky-300/30 aria-[invalid=true]:border-rose-300"
				/>
				{#if fieldError('presupuestoMaximo')}
					<span id="quick-client-presupuesto-error" class="text-xs text-rose-200">{fieldError('presupuestoMaximo')}</span>
				{/if}
			</label>

			<label class="grid gap-1.5 text-sm font-medium text-slate-200">
				Zona de interés
				<input
					id="quick-client-zonaInteres"
					bind:value={form.zonaInteres}
					onblur={() => markTouched('zonaInteres')}
					autocomplete="address-level2"
					required
					maxlength="160"
					aria-required="true"
					aria-invalid={Boolean(fieldError('zonaInteres'))}
					aria-describedby={fieldError('zonaInteres') ? 'quick-client-zona-error' : undefined}
					placeholder="Colonia, municipio o zona"
					class="rounded-lg border border-white/15 bg-slate-950 px-3 py-2.5 text-white outline-none transition focus-visible:border-sky-300 focus-visible:ring-2 focus-visible:ring-sky-300/30 aria-[invalid=true]:border-rose-300"
				/>
				{#if fieldError('zonaInteres')}
					<span id="quick-client-zona-error" class="text-xs text-rose-200">{fieldError('zonaInteres')}</span>
				{/if}
			</label>
		</div>

		<label class="flex cursor-pointer items-start gap-3 rounded-xl border border-amber-200/20 bg-amber-200/5 p-4 focus-within:ring-2 focus-within:ring-amber-200/30">
			<input
				type="checkbox"
				bind:checked={form.consentimientoEnriquecimiento}
				class="mt-1 h-4 w-4 accent-amber-300"
			/>
			<span>
				<span class="block text-sm font-medium text-amber-50">
					Consentimiento para enriquecimiento de datos (opcional)
				</span>
				<span class="mt-1 block text-xs leading-5 text-amber-100/70">
					Desactivado por defecto. Guardar este consentimiento no inicia ninguna consulta ni envía datos fuera del dispositivo.
				</span>
			</span>
		</label>

		<div class="flex flex-wrap items-center gap-3 pt-1">
			<button
				type="submit"
				class="min-h-11 rounded-lg bg-sky-400 px-5 py-2.5 text-sm font-bold text-slate-950 transition hover:bg-sky-300 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-sky-200"
			>
				Guardar lead
			</button>
			<span class="text-xs text-slate-500">Puedes navegar los campos con Tab y guardar con Enter.</span>
		</div>
	</form>
</section>

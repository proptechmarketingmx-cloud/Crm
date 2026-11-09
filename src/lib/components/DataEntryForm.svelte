<script lang="ts">
	import { onMount } from 'svelte';
	import { getCurrentWebview } from '@tauri-apps/api/webview';
	import DraftPanel from '$lib/components/DraftPanel.svelte';
	import FormField from '$lib/components/FormField.svelte';
	import ImportPanel from '$lib/components/ImportPanel.svelte';
	import {
		listLocalDrafts,
		loadClientDraft,
		loadPropertyDraft,
		saveClient,
		saveProperty,
		suggestLocations,
		type ClientFormState,
		type DraftSummary,
		type LocationSuggestion,
		type PropertyFormState
	} from '$lib/services/data-entry';

	type RecordTab = 'client' | 'property';
	type SaveStatus = 'idle' | 'dirty' | 'saving' | 'saved' | 'error';

	const EMPTY_CLIENT: ClientFormState = {
		id: null,
		nombre: '',
		apellidoPaterno: '',
		apellidoMaterno: '',
		telefono: '',
		email: '',
		curp: '',
		presupuesto: '',
		zonaInteres: '',
		tipoCredito: '',
		empresa: '',
		ingresoMensual: '',
		edad: '',
		edadLaboral: '',
		notas: '',
		consentimientos: { privacyNotice: false, osint: false }
	};

	const EMPTY_PROPERTY: PropertyFormState = {
		id: null,
		clientId: null,
		tipo: '',
		precio: '',
		recamaras: '',
		banos: '',
		niveles: '',
		calle: '',
		colonia: '',
		codigoPostal: '',
		municipio: '',
		estado: '',
		latitude: '',
		longitude: '',
		notas: '',
		documentPaths: []
	};

	let activeTab = $state<RecordTab>('client');
	let client = $state<ClientFormState>({ ...EMPTY_CLIENT, consentimientos: { ...EMPTY_CLIENT.consentimientos } });
	let property = $state<PropertyFormState>({ ...EMPTY_PROPERTY, documentPaths: [] });
	let clientDirty = $state(false);
	let propertyDirty = $state(false);
	let clientStatus = $state<SaveStatus>('idle');
	let propertyStatus = $state<SaveStatus>('idle');
	let statusMessage = $state('');
	let drafts = $state<DraftSummary[]>([]);
	let draftsError = $state('');
	let locations = $state<LocationSuggestion[]>([]);
	let locationError = $state('');
	let importError = $state('');
	let importNotice = $state('');
	let importQueue = $state<ClientFormState[]>([]);
	let importing = $state(false);
	let documentError = $state('');
	let savingClient = $state(false);
	let savingProperty = $state(false);
	let saveClientTimer: ReturnType<typeof setTimeout> | undefined;
	let savePropertyTimer: ReturnType<typeof setTimeout> | undefined;
	let locationTimer: ReturnType<typeof setTimeout> | undefined;
	let nativeDropUnlisten: (() => void) | undefined;

	const clientValidation = $derived.by(() => validateClient(client));
	const propertyValidation = $derived.by(() => validateProperty(property));
	const clientHasContent = $derived.by(() =>
		Object.entries(client).some(([key, value]) => {
			if (key === 'id' || key === 'consentimientos') return false;
			return typeof value === 'string' && value.trim().length > 0;
		}) ||
		client.consentimientos.privacyNotice ||
		client.consentimientos.osint
	);
	const propertyHasContent = $derived.by(() =>
		Object.entries(property).some(([key, value]) => {
			if (key === 'id' || key === 'clientId' || key === 'documentPaths') return false;
			return typeof value === 'string' && value.trim().length > 0;
		}) || property.documentPaths.length > 0
	);

	$effect(() => {
		if (!clientDirty || !clientHasContent || activeTab !== 'client' || savingClient) return;
		clearTimeout(saveClientTimer);
		saveClientTimer = setTimeout(() => void persistClient(true), 500);
	});

	$effect(() => {
		if (!propertyDirty || !propertyHasContent || activeTab !== 'property' || savingProperty) return;
		clearTimeout(savePropertyTimer);
		savePropertyTimer = setTimeout(() => void persistProperty(true), 500);
	});

	$effect(() => {
		const postalCode = property.codigoPostal;
		if (postalCode.length < 3 || !/^\d{3,5}$/.test(postalCode)) {
			locations = [];
			locationError = '';
			return;
		}
		clearTimeout(locationTimer);
		locationTimer = setTimeout(async () => {
			try {
				locations = await suggestLocations(postalCode);
				locationError = '';
			} catch {
				locations = [];
				locationError = 'El catálogo local de códigos postales no está disponible.';
			}
		}, 150);
	});

	onMount(() => {
		let disposed = false;
		void refreshDrafts();
		void getCurrentWebview()
			.onDragDropEvent((event) => {
				if (event.payload.type !== 'drop' || activeTab !== 'property') return;
				addDocumentPaths(event.payload.paths);
			})
			.then((unlisten) => {
				if (disposed) unlisten();
				else nativeDropUnlisten = unlisten;
			})
			.catch(() => {
				documentError = 'No se pudo activar el selector nativo de archivos.';
			});

		const onShortcut = (event: KeyboardEvent) => {
			const modifier = event.ctrlKey || event.metaKey;
			if (modifier && event.key.toLowerCase() === 's') {
				event.preventDefault();
				if (activeTab === 'client' && clientHasContent) void persistClient(true);
				else if (activeTab === 'property' && propertyHasContent) void persistProperty(true);
				else statusMessage = 'Agrega datos antes de guardar un borrador local.';
			} else if (event.altKey && event.key === '1') {
				event.preventDefault();
				activeTab = 'client';
			} else if (event.altKey && event.key === '2') {
				event.preventDefault();
				activeTab = 'property';
			}
		};
		window.addEventListener('keydown', onShortcut);

		return () => {
			disposed = true;
			nativeDropUnlisten?.();
			window.removeEventListener('keydown', onShortcut);
			clearTimeout(saveClientTimer);
			clearTimeout(savePropertyTimer);
			clearTimeout(locationTimer);
		};
	});

	function validateClient(value: ClientFormState): string[] {
		const errors: string[] = [];
		if (!value.nombre.trim()) errors.push('El nombre es obligatorio.');
		if (!value.apellidoPaterno.trim()) errors.push('El apellido paterno es obligatorio.');
		if (!/^[+()\d\s.-]{7,20}$/.test(value.telefono.trim())) errors.push('Revisa el teléfono.');
		if (value.email.trim() && !/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(value.email.trim())) {
			errors.push('Revisa el correo electrónico.');
		}
		if (value.curp.trim() && !isValidCurp(value.curp)) errors.push('La CURP no tiene un formato válido.');
		if (!value.consentimientos.privacyNotice) {
			errors.push('Confirma que informaste al cliente sobre el aviso de privacidad.');
		}
		if (!value.presupuesto.trim() || !isNonNegativeNumber(value.presupuesto)) {
			errors.push('Ingresa un presupuesto válido.');
		}
		return errors;
	}

	function validateProperty(value: PropertyFormState): string[] {
		const errors: string[] = [];
		if (!value.tipo.trim()) errors.push('Selecciona el tipo de propiedad.');
		if (!isPositiveNumber(value.precio)) errors.push('Ingresa un precio mayor a cero.');
		if (!value.calle.trim()) errors.push('Ingresa la calle y número.');
		if (!value.colonia.trim()) errors.push('Selecciona o ingresa la colonia.');
		if (!/^\d{5}$/.test(value.codigoPostal)) errors.push('El código postal debe tener 5 dígitos.');
		if (!value.municipio.trim() || !value.estado.trim()) errors.push('Completa municipio y estado.');
		if (!isCoordinate(value.latitude, -90, 90) || !isCoordinate(value.longitude, -180, 180)) {
			errors.push('Ingresa coordenadas válidas en WGS84.');
		}
		return errors;
	}

	function isValidCurp(curp: string): boolean {
		const value = curp.trim().toUpperCase();
		return /^[A-Z][AEIOUX][A-Z]{2}\d{2}(0[1-9]|1[0-2])(0[1-9]|[12]\d|3[01])[HM](AS|BC|BS|CC|CL|CM|CS|CH|DF|DG|GT|GR|HG|JC|MC|MN|MS|NT|NL|OC|PL|QT|QR|SP|SL|SR|TC|TS|TL|VZ|YN|ZS)[B-DF-HJ-NP-TV-Z]{3}[A-Z0-9]\d$/.test(value);
	}

	function isNonNegativeNumber(value: string): boolean {
		const parsed = Number(value);
		return value.trim().length > 0 && Number.isFinite(parsed) && parsed >= 0;
	}

	function isPositiveNumber(value: string): boolean {
		return isNonNegativeNumber(value) && Number(value) > 0;
	}

	function isCoordinate(value: string, minimum: number, maximum: number): boolean {
		const parsed = Number(value);
		return value.trim().length > 0 && Number.isFinite(parsed) && parsed >= minimum && parsed <= maximum;
	}

	async function persistClient(draft: boolean): Promise<void> {
		if (savingClient) return;
		if (!draft && clientValidation.length > 0) {
			statusMessage = clientValidation[0];
			return;
		}
		savingClient = true;
		clientStatus = 'saving';
		statusMessage = draft ? 'Guardando borrador cifrado…' : 'Guardando cliente cifrado…';
		const snapshot: ClientFormState = {
			...client,
			consentimientos: { ...client.consentimientos }
		};
		try {
			const response = await saveClient(snapshot, draft);
			const unchanged = JSON.stringify({ ...client, id: snapshot.id }) === JSON.stringify(snapshot);
			client.id = response.id;
			if (unchanged) {
				if (client.curp === snapshot.curp) client.curp = '';
				clientDirty = false;
			} else {
				clientDirty = true;
			}
			clientStatus = 'saved';
			statusMessage = unchanged
				? response.draft ? 'Borrador local cifrado ✓' : 'Cliente guardado localmente ✓'
				: 'Se guardó una versión local; hay cambios pendientes.';
			await refreshDrafts();
		} catch {
			clientStatus = 'error';
			statusMessage = 'No se guardó el cliente. Comprueba el almacén local cifrado.';
		} finally {
			savingClient = false;
		}
	}

	async function persistProperty(draft: boolean): Promise<void> {
		if (savingProperty) return;
		if (!draft && propertyValidation.length > 0) {
			statusMessage = propertyValidation[0];
			return;
		}
		savingProperty = true;
		propertyStatus = 'saving';
		statusMessage = draft ? 'Guardando borrador cifrado…' : 'Guardando propiedad cifrada…';
		try {
			const snapshot: PropertyFormState = {
				...property,
				documentPaths: [...property.documentPaths]
			};
			const response = await saveProperty(snapshot, draft);
			const unchanged = JSON.stringify({ ...property, id: snapshot.id }) === JSON.stringify(snapshot);
			property.id = response.id;
			propertyDirty = !unchanged;
			propertyStatus = 'saved';
			statusMessage = unchanged
				? response.draft ? 'Borrador local cifrado ✓' : 'Propiedad guardada localmente ✓'
				: 'Se guardó una versión local; hay cambios pendientes.';
			await refreshDrafts();
		} catch {
			propertyStatus = 'error';
			statusMessage = 'No se guardó la propiedad. Comprueba el almacén local cifrado.';
		} finally {
			savingProperty = false;
		}
	}

	async function refreshDrafts(): Promise<void> {
		try {
			drafts = (await listLocalDrafts()).drafts;
			draftsError = '';
		} catch {
			draftsError = 'No se pudieron consultar los borradores locales cifrados.';
		}
	}

	async function restoreDraft(draft: DraftSummary): Promise<void> {
		try {
			if (draft.kind === 'client') {
				client = await loadClientDraft(draft.id);
				clientDirty = false;
				clientStatus = 'saved';
				activeTab = 'client';
				statusMessage = 'Borrador de cliente recuperado. La CURP no se puede recuperar; introdúcela de nuevo si hace falta.';
			} else {
				property = await loadPropertyDraft(draft.id);
				propertyDirty = false;
				propertyStatus = 'saved';
				activeTab = 'property';
				statusMessage = 'Borrador de propiedad recuperado localmente.';
			}
		} catch {
			statusMessage = 'No se pudo recuperar el borrador seleccionado.';
		}
	}

	function selectLocation(colonia: string): void {
		const location = locations.find((suggestion) => suggestion.colonia === colonia);
		if (!location) return;
		property.codigoPostal = location.postalCode;
		property.colonia = location.colonia;
		property.municipio = location.municipio;
		property.estado = location.estado;
		propertyDirty = true;
	}

	function addDocumentPaths(paths: string[]): void {
		const allowed = paths.filter((path) => /\.(pdf|jpe?g|png|webp|docx?)$/i.test(path));
		const unique = new Set([...property.documentPaths, ...allowed]);
		property.documentPaths = [...unique].slice(0, 20);
		documentError = allowed.length < paths.length
			? 'Se omitieron archivos no permitidos. Se aceptan PDF, imágenes y documentos Word.'
			: '';
		propertyDirty = true;
	}

	async function readImportFile(event: Event): Promise<void> {
		importError = '';
		importNotice = '';
		importQueue = [];
		const input = event.currentTarget;
		if (!(input instanceof HTMLInputElement) || !input.files?.length) return;
		const file = input.files[0];
		if (file.size > 5_000_000) {
			importError = 'El archivo supera el límite local de 5 MB.';
			input.value = '';
			return;
		}
		try {
			const content = await file.text();
			importQueue = file.name.toLowerCase().endsWith('.vcf')
				? parseVCard(content)
				: parseCsv(content);
			if (importQueue.some((record) => record.curp && !isValidCurp(record.curp))) {
				importQueue = [];
				importError = 'El archivo contiene una CURP con formato inválido. Corrige el archivo y vuelve a importarlo.';
				return;
			}
			if (importQueue.length === 0) importError = 'No se encontraron contactos importables.';
			else importNotice = `${importQueue.length} contacto(s) listos. Se guardarán como borradores sin consentimientos concedidos.`;
		} catch {
			importError = 'No se pudo leer el archivo de forma local.';
		} finally {
			input.value = '';
		}
	}

	async function saveImportedDrafts(): Promise<void> {
		importing = true;
		const imported = [...importQueue];
		let saved = 0;
		try {
			for (const importedClient of imported) {
				await saveClient(importedClient, true);
				importedClient.curp = '';
				saved += 1;
			}
			importQueue = [];
			importNotice = `${saved} borrador(es) cifrados localmente. Revisa y confirma el aviso de privacidad antes de completar cada cliente.`;
			await refreshDrafts();
		} catch {
			importNotice = `${saved} de ${imported.length} borrador(es) guardados. La importación se detuvo por un error local; puedes reintentar los restantes.`;
			importQueue = imported.slice(saved);
			await refreshDrafts();
		} finally {
			importing = false;
		}
	}

	function parseCsv(source: string): ClientFormState[] {
		const rows = parseCsvRows(source);
		if (rows.length < 2) return [];
		const headers = rows[0].map(normalizeHeader);
		return rows.slice(1).flatMap((row) => {
			const values = Object.fromEntries(headers.map((header, index) => [header, row[index] ?? '']));
			const record = newClient();
			record.nombre = values.nombre ?? values.name ?? '';
			record.apellidoPaterno = values.apellidopaterno ?? values.paterno ?? values.lastname ?? '';
			record.apellidoMaterno = values.apellidomaterno ?? values.materno ?? '';
			record.telefono = values.telefono ?? values.phone ?? '';
			record.email = values.email ?? values.correo ?? '';
			record.curp = values.curp ?? '';
			record.presupuesto = values.presupuesto ?? values.budget ?? '';
			record.tipoCredito = values.tipocredito ?? values.credito ?? '';
			return record.nombre || record.apellidoPaterno || record.telefono ? [record] : [];
		});
	}

	function parseCsvRows(source: string): string[][] {
		const rows: string[][] = [];
		let row: string[] = [];
		let field = '';
		let quoted = false;
		for (let index = 0; index < source.length; index += 1) {
			const character = source[index];
			if (character === '"' && quoted && source[index + 1] === '"') {
				field += '"';
				index += 1;
			} else if (character === '"') {
				quoted = !quoted;
			} else if (character === ',' && !quoted) {
				row.push(field.trim());
				field = '';
			} else if ((character === '\n' || character === '\r') && !quoted) {
				if (character === '\r' && source[index + 1] === '\n') index += 1;
				row.push(field.trim());
				if (row.some((cell) => cell.length > 0)) rows.push(row);
				row = [];
				field = '';
			} else {
				field += character;
			}
		}
		row.push(field.trim());
		if (row.some((cell) => cell.length > 0)) rows.push(row);
		return rows;
	}

	function parseVCard(source: string): ClientFormState[] {
		const cards = source.replace(/\r\n[ \t]/g, '').split(/END:VCARD/i);
		return cards.flatMap((card) => {
			if (!card.trim()) return [];
			const record = newClient();
			for (const rawLine of card.split(/\r?\n/)) {
				const separator = rawLine.indexOf(':');
				if (separator < 0) continue;
				const key = rawLine.slice(0, separator).split(';')[0].toUpperCase();
				const value = unescapeVCard(rawLine.slice(separator + 1).trim());
				if (key === 'FN') {
					const [firstName = '', ...lastNames] = value.split(/\s+/);
					record.nombre ||= firstName;
					if (lastNames.length) record.apellidoPaterno ||= lastNames.join(' ');
				} else if (key === 'N') {
					const [family = '', given = '', additional = ''] = value.split(';');
					record.apellidoPaterno ||= family;
					record.nombre ||= given;
					record.apellidoMaterno ||= additional;
				} else if (key === 'TEL') record.telefono ||= value;
				else if (key === 'EMAIL') record.email ||= value;
				else if (key === 'ORG') record.empresa ||= value;
			}
			return record.nombre || record.apellidoPaterno || record.telefono ? [record] : [];
		});
	}

	function unescapeVCard(value: string): string {
		return value.replace(/\\[nN]/g, ' ').replace(/\\([,;\\])/g, '$1');
	}

	function normalizeHeader(header: string): string {
		return header.normalize('NFD').replace(/[\u0300-\u036f]/g, '').toLowerCase().replace(/[^a-z]/g, '');
	}

	function newClient(): ClientFormState {
		return { ...EMPTY_CLIENT, consentimientos: { ...EMPTY_CLIENT.consentimientos } };
	}

	function updateClient(): void {
		clientDirty = true;
		clientStatus = 'dirty';
		statusMessage = 'Cambios pendientes de guardado local.';
	}

	function updateProperty(): void {
		propertyDirty = true;
		propertyStatus = 'dirty';
		statusMessage = 'Cambios pendientes de guardado local.';
	}
</script>

<svelte:head>
	<title>CRM · Captura local</title>
</svelte:head>

<main class="mx-auto grid w-full max-w-6xl gap-5 p-4 text-slate-100 sm:p-6">
	<header class="flex flex-wrap items-start justify-between gap-4">
		<div>
			<p class="text-xs font-semibold uppercase tracking-[0.18em] text-sky-300">Captura local-first</p>
			<h1 class="mt-1 text-2xl font-bold text-white sm:text-3xl">Clientes y propiedades</h1>
			<p class="mt-2 text-sm text-slate-400">
				Los datos se guardan en la base cifrada de este dispositivo; no se envían a la nube.
			</p>
		</div>
		<nav class="flex flex-wrap gap-2" aria-label="Navegación">
			<a
				href="/quick-client"
				class="rounded-lg border border-sky-300/30 px-3 py-2 text-sm text-sky-100 hover:bg-sky-300/10 focus-visible:outline-2 focus-visible:outline-sky-300"
			>
				Alta rápida
			</a>
			<a
				href="/"
				class="rounded-lg border border-white/15 px-3 py-2 text-sm text-slate-200 hover:bg-white/5 focus-visible:outline-2 focus-visible:outline-sky-300"
			>
				Volver al matching
			</a>
		</nav>
	</header>

	<nav class="flex gap-2" aria-label="Tipo de registro">
		<button
			type="button"
			class="rounded-xl border px-4 py-2.5 text-sm font-medium focus-visible:outline-2 focus-visible:outline-sky-300 {activeTab === 'client'
				? 'border-sky-300/50 bg-sky-300/10 text-white'
				: 'border-white/10 bg-slate-900 text-slate-400'}"
			aria-pressed={activeTab === 'client'}
			onclick={() => (activeTab = 'client')}
		>
			Cliente <span class="ml-1 text-xs text-slate-400">Alt+1</span>
		</button>
		<button
			type="button"
			class="rounded-xl border px-4 py-2.5 text-sm font-medium focus-visible:outline-2 focus-visible:outline-sky-300 {activeTab === 'property'
				? 'border-sky-300/50 bg-sky-300/10 text-white'
				: 'border-white/10 bg-slate-900 text-slate-400'}"
			aria-pressed={activeTab === 'property'}
			onclick={() => (activeTab = 'property')}
		>
			Propiedad <span class="ml-1 text-xs text-slate-400">Alt+2</span>
		</button>
		<span class="ml-auto self-center text-xs text-slate-500">Guardar borrador: Ctrl/Cmd+S</span>
	</nav>

	{#if statusMessage}
		<p
			class="rounded-xl border px-4 py-3 text-sm {clientStatus === 'error' || propertyStatus === 'error'
				? 'border-rose-300/20 bg-rose-300/5 text-rose-100'
				: statusMessage.includes('✓')
					? 'border-emerald-300/20 bg-emerald-300/5 text-emerald-100'
					: 'border-sky-300/20 bg-sky-300/5 text-sky-100'}"
			role="status"
			aria-live="polite"
		>
			{statusMessage}
		</p>
	{/if}

	{#if activeTab === 'client'}
		<section class="grid gap-5 xl:grid-cols-[minmax(0,1fr)_300px]">
			<form class="grid gap-5" oninput={updateClient} onsubmit={(event) => { event.preventDefault(); void persistClient(false); }}>
				<section class="grid gap-4 rounded-2xl border border-white/10 bg-slate-900 p-4 sm:grid-cols-2 sm:p-5" aria-labelledby="client-fields-title">
					<div class="sm:col-span-2">
						<h2 id="client-fields-title" class="text-lg font-semibold text-white">Datos del cliente</h2>
						<p class="mt-1 text-xs text-slate-400">Los campos marcados con * son necesarios para completar el registro.</p>
					</div>
					<FormField label="Nombre *" bind:value={client.nombre} required autocomplete="given-name" />
					<FormField label="Apellido paterno *" bind:value={client.apellidoPaterno} required autocomplete="family-name" />
					<FormField label="Apellido materno" bind:value={client.apellidoMaterno} autocomplete="additional-name" />
					<FormField label="Teléfono *" bind:value={client.telefono} required type="tel" autocomplete="tel" />
					<FormField label="Correo" bind:value={client.email} type="email" autocomplete="email" />
					<label class="grid gap-1.5 text-sm text-slate-300">
						CURP (opcional)
						<input
							bind:value={client.curp}
							autocomplete="off"
							maxlength="18"
							spellcheck="false"
							class="rounded-lg border border-white/15 bg-slate-950 px-3 py-2.5 font-mono uppercase text-white outline-none focus-visible:border-sky-300 focus-visible:ring-2 focus-visible:ring-sky-300/30"
							aria-invalid={client.curp.length > 0 && !isValidCurp(client.curp)}
							aria-describedby="curp-help"
						/>
						<span id="curp-help" class="text-xs text-slate-500">
							Se valida localmente y Rust guarda solo un HMAC, nunca la CURP en claro.
						</span>
					</label>
					<label class="grid gap-1.5 text-sm text-slate-300">
						Crédito
						<select bind:value={client.tipoCredito} class="rounded-lg border border-white/15 bg-slate-950 px-3 py-2.5 text-white focus-visible:outline-2 focus-visible:outline-sky-300">
							<option value="">Seleccionar</option>
							<option value="infonavit">Infonavit</option>
							<option value="bancario">Bancario</option>
							<option value="cofinavit">Cofinavit</option>
							<option value="cash">Recursos propios</option>
							<option value="otro">Otro</option>
						</select>
					</label>
					<FormField label="Presupuesto (MXN) *" bind:value={client.presupuesto} type="number" min="0" step="1000" required inputmode="decimal" />
					<FormField label="Zona de interés" bind:value={client.zonaInteres} autocomplete="address-level2" />
					<FormField label="Ingreso mensual (MXN)" bind:value={client.ingresoMensual} type="number" min="0" step="500" inputmode="decimal" />
					<FormField label="Edad" bind:value={client.edad} type="number" min="18" max="120" />
					<FormField label="Antigüedad laboral (años)" bind:value={client.edadLaboral} type="number" min="0" max="100" />
					<FormField label="Empresa" bind:value={client.empresa} autocomplete="organization" />
					<label class="grid gap-1.5 text-sm text-slate-300 sm:col-span-2">
						Notas
						<textarea bind:value={client.notas} rows="3" class="resize-y rounded-lg border border-white/15 bg-slate-950 px-3 py-2.5 text-white outline-none focus-visible:border-sky-300 focus-visible:ring-2 focus-visible:ring-sky-300/30"></textarea>
					</label>
				</section>

				<section class="grid gap-4 rounded-2xl border border-white/10 bg-slate-900 p-4 sm:p-5" aria-labelledby="consent-title">
					<div>
						<h2 id="consent-title" class="text-lg font-semibold text-white">Privacidad y consentimiento</h2>
						<p class="mt-1 text-sm leading-6 text-slate-400">
							El consentimiento OSINT es opcional, separado y no se activa por defecto.
						</p>
					</div>
					<label class="flex cursor-pointer items-start gap-3 rounded-xl border border-white/10 bg-slate-950/60 p-4 focus-within:ring-2 focus-within:ring-sky-300/40">
						<input
							type="checkbox"
							bind:checked={client.consentimientos.privacyNotice}
							class="mt-1 h-4 w-4 accent-sky-300"
							required
						/>
						<span>
							<span class="block text-sm font-medium text-white">Aviso de privacidad informado *</span>
							<span class="mt-1 block text-xs leading-5 text-slate-400">
								Confirma que informaste al cliente sobre el tratamiento local de sus datos.
							</span>
						</span>
					</label>
					<label class="flex cursor-pointer items-start gap-3 rounded-xl border border-amber-200/20 bg-amber-200/5 p-4 focus-within:ring-2 focus-within:ring-amber-200/40">
						<input
							type="checkbox"
							bind:checked={client.consentimientos.osint}
							class="mt-1 h-4 w-4 accent-amber-300"
						/>
						<span>
							<span class="block text-sm font-medium text-amber-50">Consentimiento para enriquecimiento OSINT</span>
							<span class="mt-1 block text-xs leading-5 text-amber-100/70">
								No habilita búsquedas por sí solo. Fuentes públicas solo se consultan mediante una acción explícita posterior.
							</span>
						</span>
					</label>
				</section>

				<div class="flex flex-wrap items-center gap-3">
					<button type="submit" disabled={clientValidation.length > 0 || savingClient} class="rounded-lg bg-sky-400 px-4 py-2.5 text-sm font-semibold text-slate-950 hover:bg-sky-300 disabled:cursor-not-allowed disabled:opacity-40 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-sky-200">
						Guardar cliente
					</button>
					<button type="button" disabled={!clientHasContent || savingClient} onclick={() => persistClient(true)} class="rounded-lg border border-white/20 px-4 py-2.5 text-sm font-medium text-white hover:bg-white/5 disabled:cursor-not-allowed disabled:opacity-40 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-sky-200">
						Borrador local
					</button>
					{#if clientValidation.length > 0}
						<p class="text-xs text-amber-200" aria-live="polite">{clientValidation[0]}</p>
					{/if}
				</div>
			</form>

			<aside class="grid content-start gap-4">
				<ImportPanel
					{importQueue}
					{importing}
					{importError}
					{importNotice}
					onFile={readImportFile}
					onSave={saveImportedDrafts}
				/>
				<DraftPanel {drafts} {draftsError} onRestore={restoreDraft} />
			</aside>
		</section>
	{:else}
		<section class="grid gap-5 xl:grid-cols-[minmax(0,1fr)_300px]">
			<form class="grid content-start gap-5" oninput={updateProperty} onsubmit={(event) => { event.preventDefault(); void persistProperty(false); }}>
				<section class="grid gap-4 rounded-2xl border border-white/10 bg-slate-900 p-4 sm:grid-cols-2 sm:p-5" aria-labelledby="property-fields-title">
					<div class="sm:col-span-2">
						<h2 id="property-fields-title" class="text-lg font-semibold text-white">Datos de la propiedad</h2>
					</div>
					<label class="grid gap-1.5 text-sm text-slate-300">
						Tipo de inmueble *
						<select bind:value={property.tipo} required class="rounded-lg border border-white/15 bg-slate-950 px-3 py-2.5 text-white focus-visible:outline-2 focus-visible:outline-sky-300">
							<option value="">Seleccionar</option>
							<option value="casa">Casa</option>
							<option value="departamento">Departamento</option>
							<option value="terreno">Terreno</option>
							<option value="local">Local comercial</option>
							<option value="oficina">Oficina</option>
						</select>
					</label>
					<FormField label="Precio (MXN) *" bind:value={property.precio} type="number" min="1" step="1000" required inputmode="decimal" />
					<FormField label="Recámaras" bind:value={property.recamaras} type="number" min="0" max="100" />
					<FormField label="Baños" bind:value={property.banos} type="number" min="0" max="100" step="0.5" />
					<FormField label="Niveles" bind:value={property.niveles} type="number" min="0" max="100" />
					<FormField label="Calle y número *" bind:value={property.calle} required autocomplete="street-address" />
					<FormField label="Código postal *" bind:value={property.codigoPostal} required inputmode="numeric" pattern="\d{5}" maxlength={5} autocomplete="postal-code" />
					<label class="grid gap-1.5 text-sm text-slate-300">
						Colonia *
						<input
							bind:value={property.colonia}
							onchange={() => selectLocation(property.colonia)}
							list="local-colonies"
							autocomplete="address-level3"
							required
							class="rounded-lg border border-white/15 bg-slate-950 px-3 py-2.5 text-white outline-none focus-visible:border-sky-300 focus-visible:ring-2 focus-visible:ring-sky-300/30"
						/>
						<datalist id="local-colonies">
							{#each locations as location (`${location.postalCode}-${location.colonia}`)}
								<option value={location.colonia}>{location.postalCode} · {location.municipio}, {location.estado}</option>
							{/each}
						</datalist>
						{#if locationError}<span class="text-xs text-amber-200" role="status">{locationError}</span>{/if}
					</label>
					<FormField label="Municipio o alcaldía *" bind:value={property.municipio} required autocomplete="address-level2" />
					<FormField label="Estado *" bind:value={property.estado} required autocomplete="address-level1" />
					<FormField label="Latitud WGS84 *" bind:value={property.latitude} type="number" min="-90" max="90" step="any" required inputmode="decimal" />
					<FormField label="Longitud WGS84 *" bind:value={property.longitude} type="number" min="-180" max="180" step="any" required inputmode="decimal" />
					<label class="grid gap-1.5 text-sm text-slate-300 sm:col-span-2">
						Notas
						<textarea bind:value={property.notas} rows="3" class="resize-y rounded-lg border border-white/15 bg-slate-950 px-3 py-2.5 text-white outline-none focus-visible:border-sky-300 focus-visible:ring-2 focus-visible:ring-sky-300/30"></textarea>
					</label>
				</section>

				<section class="grid gap-3 rounded-2xl border border-dashed border-sky-200/25 bg-slate-900 p-4 sm:p-5" aria-labelledby="documents-title">
					<div>
						<h2 id="documents-title" class="font-semibold text-white">Documentos locales</h2>
						<p class="mt-1 text-xs leading-5 text-slate-400">
							Arrastra archivos a esta zona desde el explorador. Solo guardamos las rutas locales; los archivos no se copian ni salen del dispositivo.
						</p>
					</div>
					<div
						class="grid min-h-24 place-items-center rounded-xl border border-dashed border-white/20 bg-slate-950/50 p-4 text-center text-sm text-slate-400"
						role="region"
						aria-label="Zona para soltar documentos locales"
						ondragover={(event) => event.preventDefault()}
					>
						Suelta aquí PDF, imágenes o documentos Word · máximo 20
					</div>
					{#if property.documentPaths.length}
						<ul class="grid gap-1 text-xs text-slate-300">
							{#each property.documentPaths as path, index (`${path}-${index}`)}
								<li class="flex items-center justify-between gap-2 rounded-lg bg-slate-950 px-3 py-2">
									<span class="break-all">{path.split(/[\\/]/).at(-1)}</span>
									<button type="button" aria-label={`Quitar ${path.split(/[\\/]/).at(-1)}`} class="shrink-0 rounded px-2 py-1 text-slate-400 hover:text-white focus-visible:outline-2 focus-visible:outline-sky-300" onclick={() => { property.documentPaths = property.documentPaths.filter((_, itemIndex) => itemIndex !== index); updateProperty(); }}>
										Quitar
									</button>
								</li>
							{/each}
						</ul>
					{/if}
					{#if documentError}<p class="text-xs text-amber-200" role="status">{documentError}</p>{/if}
				</section>

				<div class="flex flex-wrap items-center gap-3">
					<button type="submit" disabled={propertyValidation.length > 0 || savingProperty} class="rounded-lg bg-sky-400 px-4 py-2.5 text-sm font-semibold text-slate-950 hover:bg-sky-300 disabled:cursor-not-allowed disabled:opacity-40 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-sky-200">
						Guardar propiedad
					</button>
					<button type="button" disabled={!propertyHasContent || savingProperty} onclick={() => persistProperty(true)} class="rounded-lg border border-white/20 px-4 py-2.5 text-sm font-medium text-white hover:bg-white/5 disabled:cursor-not-allowed disabled:opacity-40 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-sky-200">
						Borrador local
					</button>
					{#if propertyValidation.length > 0}
						<p class="text-xs text-amber-200" aria-live="polite">{propertyValidation[0]}</p>
					{/if}
				</div>
			</form>

			<aside class="grid content-start gap-4">
				<DraftPanel {drafts} {draftsError} onRestore={restoreDraft} />
			</aside>
		</section>
	{/if}
</main>

import { invoke } from '@tauri-apps/api/core';

export interface ConsentimientoFlags {
	privacyNotice: boolean;
	osint: boolean;
}

export interface ClientFormState {
	id: string | null;
	nombre: string;
	apellidoPaterno: string;
	apellidoMaterno: string;
	telefono: string;
	email: string;
	curp: string;
	presupuesto: string;
	zonaInteres: string;
	tipoCredito: string;
	empresa: string;
	ingresoMensual: string;
	edad: string;
	edadLaboral: string;
	notas: string;
	consentimientos: ConsentimientoFlags;
}

export interface PropertyFormState {
	id: string | null;
	clientId: string | null;
	tipo: string;
	precio: string;
	recamaras: string;
	banos: string;
	niveles: string;
	calle: string;
	colonia: string;
	codigoPostal: string;
	municipio: string;
	estado: string;
	latitude: string;
	longitude: string;
	notas: string;
	documentPaths: string[];
}

export interface SaveRecordResponse {
	id: string;
	draft: boolean;
	savedAt: string;
}

export interface LocationSuggestion {
	postalCode: string;
	colonia: string;
	municipio: string;
	estado: string;
}

export interface DraftSummary {
	id: string;
	kind: 'client' | 'property';
	label: string;
	updatedAt: string;
}

export interface DraftListResponse {
	drafts: DraftSummary[];
}

export async function saveClient(
	client: ClientFormState,
	draft: boolean
): Promise<SaveRecordResponse> {
	return invoke<SaveRecordResponse>('save_client_record', {
		request: { client, draft }
	});
}

export async function saveProperty(
	property: PropertyFormState,
	draft: boolean
): Promise<SaveRecordResponse> {
	return invoke<SaveRecordResponse>('save_property_record', {
		request: { property, draft }
	});
}

export function suggestLocations(postalCode: string): Promise<LocationSuggestion[]> {
	return invoke<LocationSuggestion[]>('suggest_locations', { postalCode });
}

export function listLocalDrafts(): Promise<DraftListResponse> {
	return invoke<DraftListResponse>('list_local_drafts');
}

export function loadClientDraft(id: string): Promise<ClientFormState> {
	return invoke<ClientFormState>('load_client_draft', { id });
}

export function loadPropertyDraft(id: string): Promise<PropertyFormState> {
	return invoke<PropertyFormState>('load_property_draft', { id });
}

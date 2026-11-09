import { saveClient, type ClientFormState, type SaveRecordResponse } from './data-entry';

export interface ClientPayload {
	id: string;
	nombre: string;
	apellidoPaterno: string;
	telefono: string;
	presupuestoMaximo: number;
	zonaInteres: string;
	consentimientoEnriquecimiento: boolean;
}

export type ClientField =
	| 'nombre'
	| 'apellidoPaterno'
	| 'telefono'
	| 'presupuestoMaximo'
	| 'zonaInteres';

export type FormErrors = Partial<Record<ClientField, string>>;

export function validateQuickClient(payload: Omit<ClientPayload, 'id'>): FormErrors {
	const errors: FormErrors = {};
	if (!payload.nombre.trim() || payload.nombre.length > 120) {
		errors.nombre = 'Ingresa un nombre de hasta 120 caracteres.';
	}
	if (!payload.apellidoPaterno.trim() || payload.apellidoPaterno.length > 120) {
		errors.apellidoPaterno = 'Ingresa un apellido de hasta 120 caracteres.';
	}
	const phone = payload.telefono.trim();
	if (
		phone.length > 20 ||
		!/^[+()\d\s.-]+$/.test(phone) ||
		phone.replace(/\D/g, '').length < 7
	) {
		errors.telefono = 'Ingresa un teléfono válido (7 a 20 caracteres).';
	}
	if (
		!Number.isFinite(payload.presupuestoMaximo) ||
		payload.presupuestoMaximo <= 0 ||
		String(payload.presupuestoMaximo).length > 32
	) {
		errors.presupuestoMaximo = 'Ingresa un presupuesto mayor a cero.';
	}
	if (!payload.zonaInteres.trim() || payload.zonaInteres.length > 160) {
		errors.zonaInteres = 'Indica una zona de interés de hasta 160 caracteres.';
	}
	return errors;
}

export async function saveClientLocally(payload: ClientPayload): Promise<SaveRecordResponse> {
	const client: ClientFormState = {
		id: payload.id,
		nombre: payload.nombre.trim(),
		apellidoPaterno: payload.apellidoPaterno.trim(),
		apellidoMaterno: '',
		telefono: payload.telefono.trim(),
		email: '',
		curp: '',
		presupuesto: String(payload.presupuestoMaximo),
		zonaInteres: payload.zonaInteres.trim(),
		tipoCredito: '',
		empresa: '',
		ingresoMensual: '',
		edad: '',
		edadLaboral: '',
		notas: '',
		consentimientos: {
			privacyNotice: false,
			osint: payload.consentimientoEnriquecimiento
		}
	};

	try {
		// El alta rápida crea un borrador hasta que se confirme el aviso de privacidad completo.
		return await saveClient(client, true);
	} catch {
		throw new Error('No fue posible guardar el lead en la base local cifrada.');
	}
}

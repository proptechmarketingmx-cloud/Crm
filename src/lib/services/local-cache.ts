import { invoke } from '@tauri-apps/api/core';

const DATABASE_NAME = 'crm-local-query-cache';
const DATABASE_VERSION = 1;
const STORE_NAME = 'command-results';

export interface CachedCommandResult<T> {
	value: T;
	source: 'device' | 'cache';
	storedAt: number;
	cacheWarning?: string;
}

interface CacheRecord<T> {
	key: string;
	value: T;
	storedAt: number;
}

let databasePromise: Promise<IDBDatabase> | undefined;

export async function invokeWithLocalCache<T>(
	command: string,
	args: Record<string, unknown>,
	cacheKey: string
): Promise<CachedCommandResult<T>> {
	try {
		const value = await invoke<T>(command, args);
		const storedAt = Date.now();
		try {
			await writeRecord({ key: cacheKey, value, storedAt });
			return { value, source: 'device', storedAt };
		} catch {
			return {
				value,
				source: 'device',
				storedAt,
				cacheWarning: 'No se pudo actualizar la copia IndexedDB de este resultado.'
			};
		}
	} catch (commandError) {
		try {
			const cached = await readRecord<T>(cacheKey);
			if (cached) {
				return {
					value: cached.value,
					source: 'cache',
					storedAt: cached.storedAt,
					cacheWarning: 'El comando local no está disponible; se muestra la última copia guardada.'
				};
			}
		} catch {
			throw new LocalDataUnavailableError(
				'Falló el comando local y no se pudo leer la caché IndexedDB.',
				commandError
			);
		}

		throw new LocalDataUnavailableError(
			'El comando local falló y no hay una copia guardada para esta consulta.',
			commandError
		);
	}
}

export class LocalDataUnavailableError extends Error {
	constructor(message: string, cause?: unknown) {
		super(message, { cause });
		this.name = 'LocalDataUnavailableError';
	}
}

async function openDatabase(): Promise<IDBDatabase> {
	if (typeof indexedDB === 'undefined') {
		throw new Error('IndexedDB no está disponible en este contexto.');
	}

	if (!databasePromise) {
		databasePromise = new Promise<IDBDatabase>((resolve, reject) => {
			const request = indexedDB.open(DATABASE_NAME, DATABASE_VERSION);
			request.onupgradeneeded = () => {
				const database = request.result;
				if (!database.objectStoreNames.contains(STORE_NAME)) {
					database.createObjectStore(STORE_NAME, { keyPath: 'key' });
				}
			};
			request.onsuccess = () => resolve(request.result);
			request.onerror = () => reject(request.error ?? new Error('Falló la apertura de IndexedDB.'));
			request.onblocked = () => reject(new Error('La apertura de IndexedDB está bloqueada.'));
		}).catch((error: unknown) => {
			databasePromise = undefined;
			throw error;
		});
	}

	return databasePromise;
}

async function readRecord<T>(key: string): Promise<CacheRecord<T> | undefined> {
	const database = await openDatabase();
	const transaction = database.transaction(STORE_NAME, 'readonly');
	const request = transaction.objectStore(STORE_NAME).get(key) as IDBRequest<
		CacheRecord<T> | undefined
	>;
	const [record] = await Promise.all([requestResult(request), transactionResult(transaction)]);
	return record;
}

async function writeRecord<T>(record: CacheRecord<T>): Promise<void> {
	const database = await openDatabase();
	const transaction = database.transaction(STORE_NAME, 'readwrite');
	transaction.objectStore(STORE_NAME).put(record);
	await transactionResult(transaction);
}

function requestResult<T>(request: IDBRequest<T>): Promise<T> {
	return new Promise<T>((resolve, reject) => {
		request.onsuccess = () => resolve(request.result);
		request.onerror = () => reject(request.error ?? new Error('Falló la lectura de IndexedDB.'));
	});
}

function transactionResult(transaction: IDBTransaction): Promise<void> {
	return new Promise<void>((resolve, reject) => {
		transaction.oncomplete = () => resolve();
		transaction.onerror = () =>
			reject(transaction.error ?? new Error('Falló la transacción de IndexedDB.'));
		transaction.onabort = () =>
			reject(transaction.error ?? new Error('Se canceló la transacción de IndexedDB.'));
	});
}

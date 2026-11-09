/**
 * src/lib/i18n/index.svelte.ts — Internationalization engine for 900CRM.
 *
 * Zero-dependency i18n system mirroring 900PDF's implementation exactly.
 * Supports dot-notation keys, {param} interpolation, RTL, and lazy loading.
 *
 * @module i18n
 */

import type enJson from './en.json';

// ─────────────────────────────────────────────────────────────────────────────
// Types
// ─────────────────────────────────────────────────────────────────────────────

/** Metadata stored under the `_meta` key of every locale file. */
export interface LocaleInfo {
  code: string;
  name: string;
  nativeName: string;
  direction: 'ltr' | 'rtl';
}

/**
 * Recursively flattens an object type into dot-notation string paths.
 * e.g. { nav: { dashboard: string } } → 'nav' | 'nav.dashboard'
 * The `_meta` key is excluded.
 */
type DotPaths<T, Prefix extends string = ''> = T extends object
  ? {
      [K in keyof T]: K extends '_meta'
        ? never
        : K extends string
          ? Prefix extends ''
            ? DotPaths<T[K], K> | (T[K] extends string ? K : never)
            : DotPaths<T[K], `${Prefix}.${K}`> | (T[K] extends string ? `${Prefix}.${K}` : never)
          : never;
    }[keyof T]
  : never;

/** Union of all valid translation keys, derived from English base file. */
export type TranslationKeys = DotPaths<typeof enJson>;

// eslint-disable-next-line @typescript-eslint/no-explicit-any
type TranslationDict = Record<string, any>;

/** Parameters for {placeholder} interpolation. */
export type InterpolationParams = Record<string, string | number>;

// ─────────────────────────────────────────────────────────────────────────────
// Locale registry
// ─────────────────────────────────────────────────────────────────────────────

/** All supported locales. Update when adding language files. */
export const availableLocales: LocaleInfo[] = [
  { code: 'en', name: 'English',  nativeName: 'English',    direction: 'ltr' },
  { code: 'fr', name: 'French',   nativeName: 'Français',   direction: 'ltr' },
  { code: 'es', name: 'Spanish',  nativeName: 'Español',    direction: 'ltr' },
  { code: 'ar', name: 'Arabic',   nativeName: 'العربية',    direction: 'rtl' },
  { code: 'sw', name: 'Swahili',  nativeName: 'Kiswahili',  direction: 'ltr' },
  { code: 'hi', name: 'Hindi',    nativeName: 'हिन्दी',    direction: 'ltr' },
  { code: 'pt', name: 'Portuguese (Brazil)', nativeName: 'Português (Brasil)', direction: 'ltr' },
  { code: 'vi', name: 'Vietnamese', nativeName: 'Tiếng Việt', direction: 'ltr' },
  { code: 'ha', name: 'Hausa', nativeName: 'Hausa', direction: 'ltr' },
  { code: 'bn', name: 'Bengali', nativeName: 'বাংলা', direction: 'ltr' },
];

// ─────────────────────────────────────────────────────────────────────────────
// Lazy locale loader
// ─────────────────────────────────────────────────────────────────────────────

/** Cache of loaded translation dictionaries. */
const translationCache: Map<string, TranslationDict> = new Map();

/**
 * Dynamically imports the JSON file for a locale.
 * Uses static-analysis-friendly switch so Vite can pre-compute chunks.
 */
async function loadLocale(code: string): Promise<TranslationDict | null> {
  if (translationCache.has(code)) {
    return translationCache.get(code)!;
  }

  try {
    let module: { default: TranslationDict };

    switch (code) {
      case 'en': module = await import('./en.json'); break;
      case 'fr': module = await import('./fr.json'); break;
      case 'es': module = await import('./es.json'); break;
      case 'ar': module = await import('./ar.json'); break;
      case 'sw': module = await import('./sw.json'); break;
      case 'hi': module = await import('./hi.json'); break;
      case 'pt': module = await import('./pt.json'); break;
      case 'vi': module = await import('./vi.json'); break;
      case 'ha': module = await import('./ha.json'); break;
      case 'bn': module = await import('./bn.json'); break;
      default:
        warnMissing(`Unknown locale "${code}". Falling back to English.`);
        return null;
    }

    translationCache.set(code, module.default);
    return module.default;
  } catch (err) {
    warnMissing(`Failed to load locale "${code}": ${err}`);
    return null;
  }
}

// ─────────────────────────────────────────────────────────────────────────────
// Svelte 5 reactive state
// ─────────────────────────────────────────────────────────────────────────────

/**
 * I18nStore — Svelte 5 class-based reactive state.
 * $state runes work inside .svelte.ts files and class bodies.
 */
class I18nStore {
  locale   = $state<string>('en');
  dict     = $state<TranslationDict>({});
  fallback = $state<TranslationDict>({});
  ready    = $state<boolean>(false);
}

const store = new I18nStore();

// ─────────────────────────────────────────────────────────────────────────────
// Core helpers
// ─────────────────────────────────────────────────────────────────────────────

/** Emit a dev-only warning. Silent in production. */
function warnMissing(message: string): void {
  if (import.meta.env.DEV) {
    console.warn(`[i18n] ${message}`);
  }
}

/**
 * Resolve a dot-notation key against a dictionary.
 * Returns `undefined` if the path does not exist.
 */
function resolveKey(dict: TranslationDict, key: string): string | undefined {
  const parts = key.split('.');
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  let node: any = dict;

  for (const part of parts) {
    if (node == null || typeof node !== 'object') return undefined;
    node = node[part];
  }

  return typeof node === 'string' ? node : undefined;
}

/**
 * Replace `{paramName}` placeholders with values from params.
 *
 * @example
 * interpolate('Hello {name}!', { name: 'Alice' }) → 'Hello Alice!'
 */
function interpolate(template: string, params?: InterpolationParams): string {
  if (!params) return template;
  return template.replace(/\{(\w+)\}/g, (match, key) => {
    const value = params[key];
    return value != null ? String(value) : match;
  });
}

// ─────────────────────────────────────────────────────────────────────────────
// Pluralization
// ─────────────────────────────────────────────────────────────────────────────

/**
 * Pluralization helper. Uses singular/plural key convention.
 *
 * @param key    The plural-form key
 * @param count  Numeric count determining singular vs plural
 * @param params Additional interpolation params (count auto-injected)
 */
export function tPlural(key: string, count: number, params?: InterpolationParams): string {
  const allParams = { ...params, count };
  const singularKey = deriveSingularKey(key);

  if (count === 1 && singularKey !== key) {
    const singular = resolveKey(store.dict, singularKey) ?? resolveKey(store.fallback, singularKey);
    if (singular) return interpolate(singular, allParams);
  }

  return t(key, allParams);
}

/** Derive singular key from plural by convention. */
function deriveSingularKey(pluralKey: string): string {
  if (pluralKey.endsWith('esFound')) {
    return pluralKey.replace(/esFound$/, 'Found');
  }
  const parts = pluralKey.split('.');
  const last = parts[parts.length - 1];
  if (last.endsWith('s') && !last.endsWith('ss')) {
    parts[parts.length - 1] = last.slice(0, -1);
    return parts.join('.');
  }
  return pluralKey;
}

// ─────────────────────────────────────────────────────────────────────────────
// Public API
// ─────────────────────────────────────────────────────────────────────────────

/**
 * Translate a key with optional interpolation.
 *
 * Lookup order:
 *   1. Active locale dictionary
 *   2. English fallback dictionary
 *   3. Raw key (with dev warning)
 *
 * @param key    Dot-notation key (e.g. 'nav.dashboard')
 * @param params Optional interpolation values
 * @returns      Translated, interpolated string
 */
export function t(key: string, params?: InterpolationParams): string {
  const primary = resolveKey(store.dict, key);
  if (primary !== undefined) return interpolate(primary, params);

  const fallback = resolveKey(store.fallback, key);
  if (fallback !== undefined) {
    warnMissing(`Key "${key}" missing in locale "${store.locale}", using English fallback.`);
    return interpolate(fallback, params);
  }

  warnMissing(`Key "${key}" not found in any locale dictionary.`);
  return key;
}

/**
 * Switch the active locale.
 * Loads the locale file if not yet cached.
 *
 * @param code  ISO 639-1 locale code (e.g. 'fr', 'ar')
 */
export function setLocale(code: string): void {
  _loadLocale(code);
}

/** Internal async implementation of setLocale. */
async function _loadLocale(code: string): Promise<void> {
  const supported = availableLocales.find((l) => l.code === code);
  const resolvedCode = supported ? code : 'en';

  if (!supported && code !== 'en') {
    warnMissing(`Locale "${code}" is not supported. Falling back to English.`);
  }

  store.ready = false;

  const [dict, fallbackDict] = await Promise.all([
    loadLocale(resolvedCode),
    resolvedCode !== 'en' ? loadLocale('en') : Promise.resolve(null),
  ]);

  store.locale   = resolvedCode;
  store.dict     = dict ?? {};
  store.fallback = fallbackDict ?? dict ?? {};
  store.ready    = true;

  // Update document direction for RTL languages
  if (typeof document !== 'undefined') {
    const info = availableLocales.find((l) => l.code === resolvedCode);
    document.documentElement.dir  = info?.direction ?? 'ltr';
    document.documentElement.lang = resolvedCode;
  }
}

/**
 * Returns the currently active locale code.
 * Reactive in Svelte 5 — components reading this re-render on change.
 */
export function getLocale(): string {
  return store.locale;
}

/**
 * Returns true once the active locale's dictionary has finished loading.
 */
export function isLocaleReady(): boolean {
  return store.ready;
}

/**
 * Returns the LocaleInfo record for the current locale.
 */
export function getCurrentLocaleInfo(): LocaleInfo | undefined {
  return availableLocales.find((l) => l.code === store.locale);
}

/**
 * Returns true if the active locale is right-to-left.
 *
 * @example
 * <div class:rtl={isRtl()}>…</div>
 */
export function isRtl(): boolean {
  return getCurrentLocaleInfo()?.direction === 'rtl';
}

// ─────────────────────────────────────────────────────────────────────────────
// Bootstrap
// ─────────────────────────────────────────────────────────────────────────────

/**
 * Detect the preferred locale:
 *   1. localStorage persisted preference (key: '900crm-locale')
 *   2. navigator.languages match
 *   3. Default to English
 */
function detectInitialLocale(): string {
  if (typeof localStorage !== 'undefined') {
    const saved = localStorage.getItem('900crm-locale');
    if (saved && availableLocales.some((l) => l.code === saved)) {
      return saved;
    }
  }

  if (typeof navigator !== 'undefined') {
    for (const lang of navigator.languages ?? [navigator.language]) {
      const code = lang.split('-')[0].toLowerCase();
      if (availableLocales.some((l) => l.code === code)) {
        return code;
      }
    }
  }

  return 'en';
}

/**
 * Persist the user's locale choice and apply it.
 * Call this from Settings when the user explicitly picks a language.
 *
 * @param code  ISO 639-1 locale code
 */
export function chooseLocale(code: string): void {
  if (typeof localStorage !== 'undefined') {
    localStorage.setItem('900crm-locale', code);
  }
  setLocale(code);
}

// Bootstrap: load English as fallback, then the preferred locale.
(async () => {
  const englishDict = await loadLocale('en');
  store.fallback = englishDict ?? {};

  const initialLocale = detectInitialLocale();
  await _loadLocale(initialLocale);
})();

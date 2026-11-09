/**
 * +layout.ts — SvelteKit layout configuration for 900CRM.
 * Disable SSR and enable prerendering for Tauri desktop app.
 */

/** Disable server-side rendering — Tauri runs entirely client-side. */
export const ssr = false;

/** Prerender at build time. */
export const prerender = true;

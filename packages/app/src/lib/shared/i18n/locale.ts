// shared/i18n: a thin wrapper over the generated paraglide runtime.
//
// No business logic. It exposes the available locales, the current locale, and
// a switch action that persists via cookie and reloads. The visible switcher
// widget lives elsewhere (the editor toolbar, when it exists) and imports this
// module rather than reaching into $lib/paraglide directly. Keeping one stable
// import surface decouples callers from the generated path and gives us a
// single seam to add behavior (analytics, invalidation) later.
import {
	locales as paraglideLocales,
	getLocale,
	setLocale,
	isLocale,
	type Locale
} from '$lib/paraglide/runtime';
import { m } from '$lib/paraglide/messages';

export type { Locale };

/** Display metadata for one locale, for switcher menus. */
export interface LocaleInfo {
	/** BCP 47 code used by paraglide, e.g. "en". */
	code: Locale;
	/** Human-readable, self-localized name for menus. */
	label: string;
}

/** All locales configured in project.inlang/settings.json. */
export const availableLocales: readonly Locale[] = paraglideLocales;

/** Current active locale, resolved by paraglide (cookie, then base locale). */
export function currentLocale(): Locale {
	return getLocale();
}

/** Narrowing guard re-exported so callers do not import from the runtime. */
export const isSupportedLocale: (value: unknown) => value is Locale = isLocale;

/**
 * Switch the active locale.
 *
 * Persists the PARAGLIDE_LOCALE cookie and, by default, reloads the page so SSR
 * and hydration pick up the new locale. Pass { reload: false } only when the
 * caller guarantees its own re-render. No-ops when value is not a known locale.
 */
export function switchLocale(value: unknown, options?: { reload?: boolean }): void | Promise<void> {
	if (!isLocale(value)) return;
	return setLocale(value, options);
}

/** Locales plus display labels, for rendering a switcher menu. */
export function localeOptions(): LocaleInfo[] {
	// A complete map (not Partial) makes adding a locale without its label a
	// compile error, which enforces the "how to add a locale" checklist.
	// Each label pins its own locale so it renders as that language's
	// self-name (English shows "English", de would show "Deutsch"),
	// independent of the currently active locale.
	const labels: Record<Locale, string> = {
		en: m['meta.locale_name'](undefined, { locale: 'en' })
	};
	return availableLocales.map((code) => ({ code, label: labels[code] }));
}

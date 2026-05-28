// Prerender the whole app as static HTML for @sveltejs/adapter-static.
// Locale resolves at request time from the cookie, so prerendered pages stay
// locale-agnostic until hydration. See src/lib/shared/i18n.
export const prerender = true;

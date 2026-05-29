// The theme entity: the overlay token vocabulary and the pure helper that turns
// a scene's themeId (built-in or custom) into a data-theme + inline CSS vars.

export * from './model/tokens';
export { resolveThemeStyle, type ResolvedThemeStyle } from './lib/resolve-theme-style';

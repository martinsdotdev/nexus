// The theme entity: the overlay token vocabulary and the pure helper that turns
// a scene's themeId into the inline CSS vars the themed root carries (ADR-0007).

export * from './model/tokens';
export { resolveThemeStyle, type ResolvedThemeStyle } from './lib/resolve-theme-style';
export { resolveTokenValue, wouldCycle } from './lib/link-graph';

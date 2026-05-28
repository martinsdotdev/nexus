/** A single command-palette entry. */
export interface CommandItem {
	id: string;
	label: string;
	/** Optional right-aligned hint, e.g. a keyboard shortcut. */
	hint?: string;
}

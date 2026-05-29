// The editable props per widget type, mirroring nexus-core's seed_widget_props
// (crates/nexus-core/src/default_doc.rs). Drives the inspector's per-widget form.
// Geometry, z, and visibility are universal and handled separately; these are the
// content props specific to each widget type.

export interface PropField {
	key: string;
	label: string;
	type: 'text' | 'number' | 'select';
	options?: string[];
}

export const WIDGET_PROP_SCHEMA: Record<string, PropField[]> = {
	'stream-info': [
		{ key: 'title', label: 'Title', type: 'text' },
		{ key: 'game', label: 'Game', type: 'text' }
	],
	'goal-bar': [
		{ key: 'label', label: 'Label', type: 'text' },
		{ key: 'current', label: 'Current', type: 'number' },
		{ key: 'target', label: 'Target', type: 'number' }
	],
	socials: [{ key: 'handles', label: 'Handles', type: 'text' }],
	'now-playing': [
		{ key: 'track', label: 'Track', type: 'text' },
		{ key: 'artist', label: 'Artist', type: 'text' }
	],
	'webcam-frame': [
		{ key: 'shape', label: 'Shape', type: 'select', options: ['squircle', 'rounded', 'circle'] }
	],
	'chat-box': [{ key: 'title', label: 'Title', type: 'text' }],
	alerts: [{ key: 'durationMs', label: 'Duration (ms)', type: 'number' }],
	'follower-bubble': []
};

export function propSchemaFor(widgetType: string): PropField[] {
	return WIDGET_PROP_SCHEMA[widgetType] ?? [];
}

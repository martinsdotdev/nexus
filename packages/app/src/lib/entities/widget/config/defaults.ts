// Default footprint, props, and labels per widget type for the editor's "add
// widget" affordances. Props mirror nexus-core's seed_widget_props
// (crates/nexus-core/src/default_doc.rs); the relay seeds these, the editor uses
// the same starters when a user adds a widget so behavior matches.

export const WIDGET_TYPES = [
	'webcam-frame',
	'chat-box',
	'alerts',
	'goal-bar',
	'follower-bubble',
	'stream-info',
	'now-playing',
	'socials'
] as const;

export const WIDGET_LABELS: Record<string, string> = {
	'webcam-frame': 'Webcam frame',
	'chat-box': 'Chat box',
	alerts: 'Alerts',
	'goal-bar': 'Goal bar',
	'follower-bubble': 'Follower bubble',
	'stream-info': 'Stream info',
	'now-playing': 'Now playing',
	socials: 'Socials'
};

export const DEFAULT_WIDGET_SIZE: Record<string, { w: number; h: number }> = {
	'webcam-frame': { w: 480, h: 360 },
	'chat-box': { w: 420, h: 640 },
	alerts: { w: 800, h: 200 },
	'goal-bar': { w: 520, h: 56 },
	'follower-bubble': { w: 440, h: 84 },
	'stream-info': { w: 560, h: 96 },
	'now-playing': { w: 420, h: 100 },
	socials: { w: 520, h: 48 }
};

export const DEFAULT_WIDGET_PROPS: Record<string, Record<string, unknown>> = {
	'stream-info': { title: 'My Stream', game: 'Just Chatting' },
	'goal-bar': { label: 'Follower Goal', current: 0, target: 100 },
	socials: { handles: '@nexus' },
	'now-playing': { track: 'Untitled', artist: 'Unknown Artist' },
	'webcam-frame': { shape: 'squircle' }
};

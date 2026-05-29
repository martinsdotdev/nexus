// The overlay event vocabulary (spec §9.1). Widgets subscribe to these kinds via
// the event bus; mock (and later real platform) sources emit them.

export interface ChatMessage {
	user: string;
	text: string;
	color: string;
	badges: string[];
	timestamp: number;
}
export interface FollowAlert {
	user: string;
	timestamp: number;
}
export interface SubscribeAlert {
	user: string;
	tier: number;
	months: number;
	message?: string;
}
export interface CheerAlert {
	user: string;
	bits: number;
	message?: string;
}
export interface RaidAlert {
	fromChannel: string;
	viewers: number;
}
export interface DonationAlert {
	user: string;
	amount: number;
	currency: string;
	message?: string;
}
export interface GoalIncrement {
	goalKind: string;
	by: number;
	total: number;
}
export interface TrackChanged {
	title: string;
	artist: string;
	artUrl?: string;
	source: string;
}
export interface StreamInfoChanged {
	title: string;
	game: string;
	viewerCount: number;
	uptimeSec: number;
}

export type OverlayEvent =
	| { kind: 'chat.message'; payload: ChatMessage }
	| { kind: 'alert.follow'; payload: FollowAlert }
	| { kind: 'alert.subscribe'; payload: SubscribeAlert }
	| { kind: 'alert.cheer'; payload: CheerAlert }
	| { kind: 'alert.raid'; payload: RaidAlert }
	| { kind: 'alert.donation'; payload: DonationAlert }
	| { kind: 'goal.increment'; payload: GoalIncrement }
	| { kind: 'media.track-changed'; payload: TrackChanged }
	| { kind: 'stream.info-changed'; payload: StreamInfoChanged };

export type EventKind = OverlayEvent['kind'];

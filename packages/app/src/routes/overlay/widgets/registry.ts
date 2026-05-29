// Maps a widget's `widgetType` to its renderer. Unknown types render nothing.

import type { Component } from 'svelte';
import type { WidgetProps } from './contract';
import AlertsWidget from './AlertsWidget.svelte';
import ChatBoxWidget from './ChatBoxWidget.svelte';
import FollowerBubbleWidget from './FollowerBubbleWidget.svelte';
import GoalBarWidget from './GoalBarWidget.svelte';
import NowPlayingWidget from './NowPlayingWidget.svelte';
import SocialsWidget from './SocialsWidget.svelte';
import StreamInfoWidget from './StreamInfoWidget.svelte';
import WebcamFrameWidget from './WebcamFrameWidget.svelte';

export const WIDGET_REGISTRY: Record<string, Component<WidgetProps>> = {
	'webcam-frame': WebcamFrameWidget,
	'chat-box': ChatBoxWidget,
	alerts: AlertsWidget,
	'goal-bar': GoalBarWidget,
	'follower-bubble': FollowerBubbleWidget,
	'now-playing': NowPlayingWidget,
	'stream-info': StreamInfoWidget,
	socials: SocialsWidget
};

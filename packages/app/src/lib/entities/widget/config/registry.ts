// Maps a widget's `widgetType` to its renderer. Unknown types render nothing.

import type { Component } from 'svelte';
import type { WidgetProps } from '../model/contract';
import AlertsWidget from '../ui/AlertsWidget.svelte';
import ChatBoxWidget from '../ui/ChatBoxWidget.svelte';
import FollowerBubbleWidget from '../ui/FollowerBubbleWidget.svelte';
import GoalBarWidget from '../ui/GoalBarWidget.svelte';
import NowPlayingWidget from '../ui/NowPlayingWidget.svelte';
import SocialsWidget from '../ui/SocialsWidget.svelte';
import StreamInfoWidget from '../ui/StreamInfoWidget.svelte';
import WebcamFrameWidget from '../ui/WebcamFrameWidget.svelte';

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

// Maps a widget's `widgetType` to its renderer. Unknown types render nothing.
// Extended as widgets are added.

import type { Component } from 'svelte';
import type { WidgetProps } from './contract';
import StreamInfoWidget from './StreamInfoWidget.svelte';

export const WIDGET_REGISTRY: Record<string, Component<WidgetProps>> = {
	'stream-info': StreamInfoWidget
};

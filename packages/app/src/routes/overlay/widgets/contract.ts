// The overlay widget contract. Every widget renders inside its bounding box from
// its instance (geometry + props) and may subscribe to the event bus. Widgets
// style ONLY via theme CSS variables (var(--token)); no editor tokens, no
// hardcoded colors. Co-located with the overlay route (its only consumer).

import type { WidgetView } from '$lib/shared/crdt/workspace-view';
import type { EventBus } from '$lib/shared/events/event-bus';

export interface WidgetProps {
	instance: WidgetView;
	bus: EventBus;
}

// The widget entity: the overlay widget components, their type registry, and the
// render contract. Consumed by both /overlay and the editor canvas.

export { WIDGET_REGISTRY } from './config/registry';
export { WIDGET_PROP_SCHEMA, propSchemaFor, type PropField } from './config/prop-schema';
export type { WidgetProps } from './model/contract';

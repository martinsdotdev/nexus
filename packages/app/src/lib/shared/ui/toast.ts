// The editor's toast channel: one Ark UI toaster store, shared across the editor chrome, with
// success / error / info helpers. Features publish through `toast.*`; the `<Toaster>` component
// (mounted once per app page) renders whatever is in the store. Editor chrome only, the
// `/overlay` bundle never imports this (it must stay Ark-free).
import { createToaster } from '@ark-ui/svelte/toast';

export const toaster = createToaster({
	placement: 'bottom-end',
	overlap: true,
	gap: 12,
	max: 4
});

type Kind = 'success' | 'error' | 'info';

const emit = (type: Kind, title: string, description?: string) =>
	toaster.create({ type, title, description });

export const toast = {
	success: (title: string, description?: string) => emit('success', title, description),
	error: (title: string, description?: string) => emit('error', title, description),
	info: (title: string, description?: string) => emit('info', title, description)
};

import { redirect } from '@sveltejs/kit';
import type { LayoutLoad } from './$types.js';

export const load: LayoutLoad = () => {
	const token = localStorage.getItem('ZEPHYR_AUTH');

	if (token == null) {
		redirect(307 /* Temporary Redirect */, '/login');
	}
};

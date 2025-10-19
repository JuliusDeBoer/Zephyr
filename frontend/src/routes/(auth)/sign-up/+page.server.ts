import type { Actions, PageServerLoad } from './$types.js';
import { fail } from '@sveltejs/kit';
import { superValidate } from 'sveltekit-superforms';
import { schema } from './schema';
import { zod } from 'sveltekit-superforms/adapters';

export const load: PageServerLoad = async () => {
	return {
		form: await superValidate(zod(schema))
	};
};

export const actions: Actions = {
	default: async (event) => {
		const form = await superValidate(event, zod(schema));
		if (!form.valid) {
			return fail(400, {
				form
			});
		}

		const result = await fetch('http://localhost:3000/auth/sign-up', {
			method: 'POST',
			body: JSON.stringify(form.data)
		});

		console.debug(result);

		return {
			form
		};
	}
};

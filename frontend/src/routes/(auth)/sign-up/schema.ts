import { z } from 'zod/v3';

export const schema = z.object({
	email: z.string().email().nonempty(),
	password: z.string().nonempty().min(6),

	firstName: z.string().nonempty(),
	affix: z.string().optional(),
	lastName: z.string().nonempty()
});

export type Schema = typeof schema;

import { z } from 'zod/v3';

export const schema = z.object({
	email: z.string().email().nonempty(),
	password: z.string().nonempty()
});

export type Schema = typeof schema;

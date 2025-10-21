<script lang="ts">
	import { Input } from '$lib/components/ui/input';
	import * as Form from '$lib/components/ui/form/index.js';
	import { Button } from '$lib/components/ui/button';
	import { zodClient } from 'sveltekit-superforms/adapters';
	import { type PageData } from './$types';
	import { schema } from './schema';
	import { superForm } from 'sveltekit-superforms';

	let { data }: { data: PageData } = $props();

	const form = superForm(data.form, {
		validators: zodClient(schema),
		onResult: (result) => {
			console.debug(result);
			// @ts-expect-error I cannot be bothered to actaully fix the type
			localStorage.setItem('ZEPHYR_AUTH', result.result.data.token);
			location.href = '/cal';
		}
	});

	const { form: formData, enhance } = form;
</script>

<h2 class="mb-8 scroll-m-20 text-4xl font-extrabold tracking-tight lg:text-5xl">Login</h2>
<form method="POST" use:enhance class="gap-4">
	<Form.Field {form} name="email">
		<Form.Control>
			{#snippet children({ props })}
				<Input {...props} bind:value={$formData.email} type="email" placeholder="Email Address" />
			{/snippet}
		</Form.Control>
		<Form.FieldErrors />
	</Form.Field>
	<Form.Field {form} name="password">
		<Form.Control>
			{#snippet children({ props })}
				<Input {...props} bind:value={$formData.password} type="password" placeholder="Password" />
			{/snippet}
		</Form.Control>
		<Form.FieldErrors />
	</Form.Field>
	<Button class="mt-4 w-full" type="submit">Login</Button>
</form>
<div class="mt-3 flex w-full items-center gap-4">
	<hr class="flex-grow bg-muted" />
	<p class="w-max">Dont have an account?</p>
	<hr class="flex-grow bg-muted" />
</div>
<Button variant="outline" class="w-full" href="/sign-up">Sign Up</Button>

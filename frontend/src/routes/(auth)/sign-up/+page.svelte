<script lang="ts">
	import { Input } from '$lib/components/ui/input';
	import * as Form from '$lib/components/ui/form/index.js';
	import { Button } from '$lib/components/ui/button';
	import { schema } from './schema';
	import { superForm } from 'sveltekit-superforms';
	import { zodClient } from 'sveltekit-superforms/adapters';
	import { type PageData } from './$types';

	let { data }: { data: PageData } = $props();

	const form = superForm(data.form, {
		validators: zodClient(schema)
	});

	const { form: formData, enhance } = form;
</script>

<h2 class="mb-8 scroll-m-20 text-4xl font-extrabold tracking-tight lg:text-5xl">Sign Up</h2>
<form method="POST" use:enhance class="gap-4">
	<Form.Field {form} name="firstName">
		<Form.Control>
			{#snippet children({ props })}
				<Input {...props} bind:value={$formData.firstName} type="text" placeholder="First Name" />
			{/snippet}
		</Form.Control>
		<Form.FieldErrors />
	</Form.Field>

	<div class="grid w-full gap-4 md:grid-cols-2">
		<Form.Field {form} name="affix">
			<Form.Control>
				{#snippet children({ props })}
					<Input
						{...props}
						bind:value={$formData.affix}
						type="text"
						placeholder="Affix (Optional)"
					/>
				{/snippet}
			</Form.Control>
			<Form.FieldErrors />
		</Form.Field>
		<Form.Field {form} name="lastName">
			<Form.Control>
				{#snippet children({ props })}
					<Input {...props} bind:value={$formData.lastName} type="text" placeholder="Last Name" />
				{/snippet}
			</Form.Control>
			<Form.FieldErrors />
		</Form.Field>
	</div>
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
	<Button class="mt-4 w-full" type="submit">Sign Up</Button>
</form>
<div class="mt-3 flex w-full items-center gap-4">
	<hr class="flex-grow bg-muted" />
	<p class="w-max">Already have an account?</p>
	<hr class="flex-grow bg-muted" />
</div>
<Button variant="outline" class="w-full" href="/login">Login</Button>

<script lang="ts">
	import '../app.css';
	import favicon from '$lib/assets/favicon.svg';
	import { ModeWatcher } from 'mode-watcher';

	let { children } = $props();
</script>

<svelte:head>
	<title>Zephyr Calendar</title>
	<link rel="icon" href={favicon} />
</svelte:head>

<ModeWatcher />

<div
	style="padding-top: env(safe-area-inset-top, 0); padding-bottom: env(safe-area-inset-bottom, 0);"
>
	{@render children?.()}
</div>

<script lang="ts">
	import * as Sidebar from '$lib/components/ui/sidebar';
	let { children } = $props();
</script>

<Sidebar.Provider>
	<Sidebar.Root>
		<Sidebar.Header>
			<h1
				class="scroll-m-20 pb-2 text-3xl font-semibold tracking-tight transition-colors first:mt-0"
			>
				Zephyr
			</h1>
		</Sidebar.Header>
	</Sidebar.Root>
	<Sidebar.Inset>
		{@render children?.()}
	</Sidebar.Inset>
</Sidebar.Provider>

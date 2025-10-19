<script lang="ts">
	import Grass from '$lib/assets/grass.jpg?enhanced';

	let { children } = $props();
</script>

<div class="relative min-h-screen">
	<enhanced:img src={Grass} alt="" sizes="100vw" class="absolute h-screen w-screen object-cover" />
	<div class="relative flex min-h-screen p-12">
		<div
			class="flex w-full grow grid-cols-2 grid-rows-1 flex-col overflow-hidden rounded-xl shadow-lg lg:grid"
		>
			<div
				class="flex h-[20vh] flex-col items-center justify-center gap-8 bg-[#0002] backdrop-blur-lg lg:h-auto"
			>
				<h1 class="tracking-center text-center text-6xl font-extrabold lg:text-7xl">
					Zephyr
					<br />
					<span class="hidden lg:inline">Calendar</span>
				</h1>
				<p class="mx-4 hidden text-center text-slate-200 italic lg:block">
					Because we needed a calendar application that fails whenever it wants.
				</p>
			</div>
			<div class="flex grow flex-col items-center bg-background p-8 pt-24 lg:justify-center">
				<div class="flex w-[60%] flex-col items-center justify-center">
					{@render children?.()}
				</div>
			</div>
		</div>
	</div>
</div>

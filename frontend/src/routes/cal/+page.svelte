<div class="flex min-h-12 w-full items-center justify-between px-6 py-1">
	<p>TODO</p>
	<h1 class="scroll-m-20 pb-2 text-3xl font-semibold tracking-tight transition-colors first:mt-0">
		November
	</h1>
	<p>Account</p>
</div>
<div class="grid min-h-full grid-cols-7 grid-rows-5">
	{#each Array(35), i (i)}
		<div class="border">
			<p class="m-3 w-8 text-right font-mono text-lg">{1 + ((27 + i) % 31)}</p>
		</div>
	{/each}
</div>

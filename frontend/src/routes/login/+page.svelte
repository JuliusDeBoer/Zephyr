<script lang="ts">
	import { Input } from '$lib/components/ui/input';
	import { Button } from '$lib/components/ui/button';
	import Grass from '$lib/assets/grass.jpg?enhanced';
</script>

<div class="relative min-h-screen">
	<enhanced:img src={Grass} alt="" sizes="100vw" class="absolute h-screen w-screen object-cover" />
	<div class="relative flex min-h-screen p-12">
		<div class="grid w-full grow grid-cols-2 grid-rows-1 overflow-hidden rounded-xl shadow-lg">
			<div class="flex flex-col items-center justify-center gap-8 bg-[#0002] backdrop-blur-lg">
				<h1 class="tracking-center text-center text-7xl font-extrabold">
					Zephyr
					<br />
					Calendar
				</h1>
				<p class="text-slate-200 italic">
					Because we needed a calendar application that fails whenever it wants.
				</p>
			</div>
			<div class="flex flex-col items-center justify-center bg-background p-8">
				<div class="flex w-[60%] flex-col items-center justify-center gap-4">
					<h2 class="mb-8 scroll-m-20 text-4xl font-extrabold tracking-tight lg:text-5xl">Login</h2>
					<Input type="email" placeholder="Email Adress" />
					<Input type="password" placeholder="Password" />
					<Button color="primary" class="mt-4 w-full">Login</Button>
				</div>
			</div>
		</div>
	</div>
</div>

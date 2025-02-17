<script lang="ts">
	import '../app.css';
	import { page } from '$app/state';

	let { children } = $props();

	const links = [
		{ name: 'Real-time', href: '/real-time' },
		{ name: 'Flow', href: '/flow' },
		// { name: 'Trace', href: '/trace' },
		{ name: 'JSON', href: '/json' }
	];
</script>

<div class="flex h-full flex-col">
	<nav class="bg-primary-foreground h-12 border-b">
		<div class="flex h-full items-center gap-8 px-4">
			<span class="mr-16 font-semibold text-muted-foreground">Optic Nerve</span>
			{#each links as link}
				<a
					href={link.href}
					class="flex h-full items-center {page.url.pathname === link.href
						? 'text-primary'
						: 'text-muted-foreground'} hover:text-primary">{link.name}</a
				>
			{/each}
		</div>
	</nav>
	<div class="flex-1 p-4 overflow-hidden">
		{@render children()}
	</div>
</div>

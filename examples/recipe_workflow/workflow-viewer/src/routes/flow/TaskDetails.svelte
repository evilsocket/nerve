<script lang="ts">
	import data from '$lib/data/data.json';
    import { fade, slide } from 'svelte/transition';
	import { JsonView } from '@zerodevx/svelte-json-view';
	import { marked } from 'marked';
	import { flowState } from './state.svelte';

	let selectedNodeId: string | null = $state(null);
	let selectedMessage: any = $state(null);
	let selectedData = $derived.by(() => {
		if (!flowState.activeNode) {
			return data;
		} else {
			return data.filter((node) => node.id === flowState.activeNode);
		}
	});

	function selectMessage(message: any) {
		selectedMessage = message;
	}
</script>

{#snippet messageTab(message: string, type: string, nodeId: string)}
	<div
		class="hover:bg-primary-foreground w-full cursor-pointer rounded-md border-b p-1 pl-6 text-left focus:outline-none {selectedMessage ===
		message
			? 'bg-primary-foreground'
			: ''}"
		onclick={() => {
			selectedNodeId = nodeId;
			selectMessage(message);
		}}
		onkeydown={(e) => {
			if (e.key === 'Enter' || e.key === ' ') {
				selectedNodeId = nodeId;
				selectMessage(message);
			}
		}}
		role="tab"
		tabindex={0}
		aria-selected={selectedMessage === message}
		aria-controls="message-content"
	>
		<p class="text-muted-foreground text-sm">{type}</p>
		<p class="truncate text-sm">{message.substring(0, 50)}...</p>
	</div>
{/snippet}

<div class="flex h-full">
	<div class="border-border w-3/12 border-r">
		<h2 class="border-b py-2 pl-4 text-muted-foreground font-semibold">Events</h2>
		<div class="h-full overflow-y-auto pb-12">
			{#each selectedData as node}
				<div class="">
					<h3 class="text-md font-semibold border-b px-4 py-4">{node.name}</h3>
					{@render messageTab(node.chat.system_prompt, 'system', node.id)}
					{@render messageTab(node.chat.prompt, 'user', node.id)}
					{#each node.chat.history.conversation as message}
						{@render messageTab(message.data[0], message.type, node.id)}
					{/each}
				</div>
			{/each}
		</div>
	</div>

	<!-- Markdown Content -->
	<div class="border-border w-6/12 border-r">
		<h2 class="border-b py-2 pl-4 text-muted-foreground  font-semibold">Data</h2>
		<div class="h-full overflow-y-auto">
			{#if selectedMessage}
				<div class="prose prose-invert max-w-none p-4">
					{@html marked(selectedMessage)}
				</div>
			{:else}
				<p class="text-muted-foreground p-4">Select an event to view content</p>
			{/if}
		</div>
	</div>

	<!-- State Viewer -->
	<div class="w-3/12">
		<h2 class="border-b py-2 pl-4 text-muted-foreground  font-semibold">State</h2>
		<div class="h-full overflow-y-auto">
			{#if selectedNodeId}
				<div class="jsonView p-4">
					<h3 class="text-sm mb-1 text-muted-foreground  font-medium">Globals</h3>
					<JsonView json={data.find((n) => n.id === selectedNodeId)?.globals || {}} />
				</div>
				<div class="jsonView p-4">
					<h3 class="text-sm mb-1 text-muted-foreground font-medium">Variables</h3>
					<JsonView json={data.find((n) => n.id === selectedNodeId)?.variables || {}} />
				</div>
			{:else}
				<p class="text-muted-foreground p-4">Select an event to view state</p>
			{/if}
		</div>
	</div>
</div>

<style>
	:global(.prose) {
		color: inherit;
	}
	:global(ul) {
		padding-left: var(--jsonPaddingLeft, 1.25rem) !important;
	}
	.jsonView {
		scrollbar-color: rgb(15, 23, 42) transparent;
		scrollbar-width: thick;
		overflow-y: auto;
		list-style-type: none;
		--jsonValStringColor: #aaaaaa;
		/* padding-left: var(--jsonPaddingLeft, 1rem); */
	}
</style>

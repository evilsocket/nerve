<script lang="ts">
	import { JsonView } from '@zerodevx/svelte-json-view';
	import data from '$lib/data/data.json';
	import { fade } from 'svelte/transition';
</script>

<div class="jsonView p-6 h-full border" in:fade>
	<JsonView json={data} depth={1} />
</div>

<style>
	:global(ul) {
		padding-left: var(--jsonPaddingLeft, 1.25rem)!important;
	}
	.jsonView {
        overflow-y: auto;
		list-style-type: none;
        --jsonValStringColor: #aaaaaa;
		/* padding-left: var(--jsonPaddingLeft, 1rem); */
	}
</style>

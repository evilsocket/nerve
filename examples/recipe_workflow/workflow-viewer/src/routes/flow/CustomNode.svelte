<script lang="ts">
	import { Handle, Position, type NodeProps } from '@xyflow/svelte';
	import { flowState } from './state.svelte';

	let { id, data }: NodeProps = $props();
</script>

<Handle type="target" position={Position.Left} />
<div
	class="bg-primary-foreground flex h-[100px] w-[175px] flex-col justify-center gap-1 rounded-3xl border p-2 text-center ring-sky-500 hover:ring-1 {flowState.activeNode === id ? 'ring-1' : ''}"
>
	<span class="text-[10px] text-muted-foreground">Task ID: {id}</span>
	<span class="text-xs">{data.label}</span>
</div>
<Handle type="source" position={Position.Right} />

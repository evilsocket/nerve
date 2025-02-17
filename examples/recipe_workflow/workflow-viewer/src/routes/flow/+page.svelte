<script lang="ts">
	import { SvelteFlow, Background, Controls, type ColorMode, MarkerType } from '@xyflow/svelte';
	import { type Node, type Edge, Position } from '@xyflow/svelte';
	import data from '$lib/data/data.json';
	import '@xyflow/svelte/dist/style.css';
	import CustomNode from './CustomNode.svelte';
	import { flowState } from './state.svelte';
	import TaskDetails from './TaskDetails.svelte';
	import { fade } from 'svelte/transition';

	let selectedNode = $state('');
	let nodes = $state.raw<Node[]>(
		data.map((node, i) => ({
			id: node.id,
			type: 'customNode',
			position: { x: i * 300, y: 0 },
			data: { label: node.name, active: node.id === selectedNode },
			sourcePosition: Position.Right,
			targetPosition: Position.Left
		}))
	);
	let edges = $state.raw<Edge[]>([
		{
			id: '1-2',
			source: '1',
			target: '2',
			markerEnd: {
				type: MarkerType.Arrow
			}
		},
		{
			id: '2-3',
			source: '2',
			target: '3',
			markerEnd: {
				type: MarkerType.Arrow
			}
		},
		{
			id: '3-4',
			source: '3',
			target: '4',
			markerEnd: {
				type: MarkerType.Arrow
			}
		}
	]);
</script>

<div class="flex h-full flex-col border" in:fade>
	<div class="flex-[40%] border-b">
		<SvelteFlow
			bind:nodes
			bind:edges
			nodeTypes={{ customNode: CustomNode }}
			colorMode="dark"
			fitView
			nodesDraggable={false}
			nodesConnectable={false}
			onnodeclick={(e) => {
				if (flowState.activeNode === e.node.id) {
					flowState.activeNode = '';
				} else {
					flowState.activeNode = e.node.id;
				}
			}}
		>
			<Background bgColor="rgb(2, 8, 23)" patternColor="rgb(148, 163, 184)" size={0.5} />
			<Controls showLock={false} />
		</SvelteFlow>
	</div>
	<div class="flex-[60%] overflow-hidden">
		<TaskDetails />
	</div>
</div>

<script lang="ts">
	import Map from '$lib/components/Map.svelte';
	import CommandBar from '$lib/components/CommandBar.svelte';
	import Sidebar from '$lib/components/Sidebar.svelte';
	import TableView from '$lib/components/TableView.svelte';
</script>

<div class="app-shell">
	<Sidebar />
	<div class="workspace">
		<div class="map-pane">
			<Map />
		</div>
		<div class="table-pane">
			<TableView />
			<CommandBar />
		</div>
	</div>
</div>

<style>
	.app-shell {
		position: relative;
		width: 100vw;
		height: 100vh;
		overflow: hidden;
	}

	.workspace {
		position: absolute;
		top: 10px;
		right: 10px;
		bottom: 10px;
		left: 342px;
		display: grid;
		grid-template-rows: minmax(0, 1fr) minmax(0, 1fr);
		border: 1px solid rgba(255, 255, 255, 0.08);
		border-radius: 12px;
		overflow: hidden;
		background: rgba(8, 12, 18, 0.82);
	}

	.map-pane,
	.table-pane {
		position: relative;
		min-height: 0;
	}

	.map-pane {
		background: #0a0e17;
	}

	.table-pane {
		background: rgba(15, 20, 30, 0.96);
	}

	@media (max-width: 900px) {
		.workspace {
			left: 10px;
			top: 360px;
		}
	}
</style>

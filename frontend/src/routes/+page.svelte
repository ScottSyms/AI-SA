<script lang="ts">
	import Map from '$lib/components/Map.svelte';
	import DetailPanel from '$lib/components/DetailPanel.svelte';
	import CommandBar from '$lib/components/CommandBar.svelte';
	import TableView from '$lib/components/TableView.svelte';
	import Toolbar from '$lib/components/Toolbar.svelte';
</script>

<div class="app-shell">
	<Map />
	<Toolbar />
	<DetailPanel />
	<TableView />
	<CommandBar />
</div>

<style>
	.app-shell {
		position: relative;
		width: 100vw;
		height: 100vh;
		overflow: hidden;
	}
</style>

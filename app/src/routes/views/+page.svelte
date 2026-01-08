<script lang="ts">
	import type { PageData } from './$types';

	export const data: PageData = {};
</script>

<div class="page">
	<header class="page-header">
		<div class="header-content">
			<h1>Views</h1>
			<p>Navigate your architecture through multiple perspectives and custom views</p>
		</div>
	</header>

	<section class="views-container">
		<div class="placeholder">
			<h2>📊 Views Coming Soon</h2>
			<p>This feature will allow you to:</p>
			<ul>
				<li>Create custom views of your architecture</li>
				<li>Filter cards by type, status, and relationships</li>
				<li>Save and reuse view configurations</li>
				<li>Generate reports and exports</li>
			</ul>
		</div>
	</section>
</div>

<style>
	.page {
		display: flex;
		flex-direction: column;
		height: 100%;
	}

	.page-header {
		background: linear-gradient(135deg, var(--md-sys-color-primary) 0%, var(--md-sys-color-secondary) 100%);
		color: var(--md-sys-color-on-primary);
		padding: 2rem;
		border-radius: 12px;
		margin-bottom: 2rem;
	}

	.header-content h1 {
		font-size: 2rem;
		font-weight: 500;
		margin: 0 0 0.5rem 0;
	}

	.header-content p {
		margin: 0;
		opacity: 0.9;
	}

	.views-container {
		display: flex;
		align-items: center;
		justify-content: center;
		flex: 1;
		min-height: 400px;
	}

	.placeholder {
		text-align: center;
		background-color: var(--md-sys-color-surface-container);
		border: 2px dashed var(--md-sys-color-outline-variant);
		border-radius: 12px;
		padding: 3rem 2rem;
		max-width: 600px;
	}

	.placeholder h2 {
		font-size: 1.5rem;
		color: var(--md-sys-color-primary);
		margin-top: 0;
	}

	.placeholder p {
		color: var(--md-sys-color-on-surface-variant);
		margin-bottom: 1.5rem;
	}

	.placeholder ul {
		text-align: left;
		display: inline-block;
		color: var(--md-sys-color-on-surface-variant);
	}

	.placeholder li {
		margin-bottom: 0.75rem;
	}
</style>

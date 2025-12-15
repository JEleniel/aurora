<script lang="ts">
	export let id: string = '';
	export let name: string = '';
	export let label: string = '';
	export let value: string = '';
	export let placeholder: string = '';
	export let required: boolean = false;
	export let disabled: boolean = false;
	export let rows: number = 4;

	function handleChange(e: Event) {
		value = (e.target as HTMLTextAreaElement).value;
	}
</script>

<div class="form-field">
	{#if label}
		<label for={id}>{label}</label>
	{/if}
	<textarea
		{id}
		{name}
		{value}
		{placeholder}
		{required}
		{disabled}
		{rows}
		on:change={handleChange}
		on:input={handleChange}
	></textarea>
</div>

<style>
	.form-field {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
		margin-bottom: 1.5rem;
	}

	label {
		font-size: 0.875rem;
		font-weight: 500;
		color: var(--md-sys-color-on-surface);
	}

	textarea {
		padding: 0.75rem;
		border: 1px solid var(--md-sys-color-outline);
		border-radius: 4px;
		background-color: var(--md-sys-color-surface);
		color: var(--md-sys-color-on-surface);
		font-size: 0.875rem;
		font-family: inherit;
		resize: vertical;
		transition: all 0.2s ease;
	}

	textarea:focus {
		outline: none;
		border-color: var(--md-sys-color-primary);
		box-shadow: 0 0 0 3px color-mix(in srgb, var(--md-sys-color-primary) 12%, transparent);
	}

	textarea:disabled {
		opacity: 0.38;
		cursor: not-allowed;
	}
</style>

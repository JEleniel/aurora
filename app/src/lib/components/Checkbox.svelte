<script lang="ts">
	export let id: string = '';
	export let name: string = '';
	export let label: string = '';
	export let checked: boolean = false;
	export let disabled: boolean = false;

	function handleChange(e: Event) {
		checked = (e.target as HTMLInputElement).checked;
	}
</script>

<label class="checkbox-wrapper">
	<input {id} {name} type="checkbox" {checked} {disabled} on:change={handleChange} />
	<span class="checkbox-label">{label}</span>
</label>

<style>
	.checkbox-wrapper {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		cursor: pointer;
		font-size: 0.875rem;
		color: var(--md-sys-color-on-surface);
		margin-bottom: 0.75rem;
	}

	input[type='checkbox'] {
		width: 20px;
		height: 20px;
		cursor: pointer;
		accent-color: var(--md-sys-color-primary);
	}

	input[type='checkbox']:disabled {
		opacity: 0.38;
		cursor: not-allowed;
	}

	.checkbox-wrapper:has(input:disabled) {
		opacity: 0.38;
		cursor: not-allowed;
	}

	.checkbox-label {
		flex: 1;
	}
</style>

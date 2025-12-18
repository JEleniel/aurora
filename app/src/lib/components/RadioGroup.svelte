<script lang="ts">
	let {
		name = '',
		label = '',
		value = $bindable(''),
		options = [],
		required = false,
		disabled = false,
		onchange = undefined,
	} = $props();

	function handleChange(e: Event) {
		value = (e.target as HTMLInputElement).value;
		onchange?.(e);
	}
</script>

<fieldset class="radio-group">
	{#if label}
		<legend>{label}</legend>
	{/if}
	<div class="options">
		{#each options as option}
			<label class="radio-option">
				<input
					type="radio"
					{name}
					value={option.value}
					checked={value === option.value}
					{required}
					{disabled}
					onchange={handleChange}
				/>
				<span>{option.label}</span>
			</label>
		{/each}
	</div>
</fieldset>

<style>
	.radio-group {
		border: none;
		padding: 0;
		margin-bottom: 1.5rem;
	}

	legend {
		font-size: 0.875rem;
		font-weight: 500;
		color: var(--md-sys-color-on-surface);
		padding: 0 0 0.5rem 0;
	}

	.options {
		display: flex;
		flex-direction: column;
		gap: 0.75rem;
	}

	.radio-option {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		cursor: pointer;
		font-size: 0.875rem;
		color: var(--md-sys-color-on-surface);
	}

	input[type='radio'] {
		width: 20px;
		height: 20px;
		cursor: pointer;
		accent-color: var(--md-sys-color-primary);
	}

	input[type='radio']:disabled {
		opacity: 0.38;
		cursor: not-allowed;
	}

	.radio-option:has(input:disabled) {
		opacity: 0.38;
		cursor: not-allowed;
	}
</style>

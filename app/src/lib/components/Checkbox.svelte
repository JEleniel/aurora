<script lang="ts">
	let {
		id = '',
		name = '',
		label = '',
		checked = $bindable(false),
		disabled = false,
		required = false,
		helpText = '',
		ariaDescribedBy = '',
		onchange = undefined,
	} = $props();

	let helpId = $derived(id ? `${id}-help` : '');
	let combinedDescribedBy = $derived([ariaDescribedBy, helpText ? helpId : ''].filter(Boolean).join(' '));

	function handleChange(e: Event) {
		checked = (e.target as HTMLInputElement).checked;
		onchange?.(e);
	}
</script>

<div class="checkbox-group">
	<label class="checkbox-wrapper">
		<input
			{id}
			{name}
			type="checkbox"
			{checked}
			{disabled}
			{required}
			aria-required={required}
			aria-describedby={combinedDescribedBy || undefined}
			onchange={handleChange}
		/>
		<span class="checkbox-label">{label}</span>
	</label>
	{#if helpText}
		<small class="help-text" id={helpId}>{helpText}</small>
	{/if}
</div>

<style>
	.checkbox-group {
		margin-bottom: 0.75rem;
	}

	.checkbox-wrapper {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		cursor: pointer;
		font-size: 0.875rem;
		color: var(--md-sys-color-on-surface);
	}

	.checkbox-wrapper:has(input:focus) {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
		border-radius: 2px;
	}

	input[type='checkbox'] {
		width: 20px;
		height: 20px;
		cursor: pointer;
		accent-color: var(--md-sys-color-primary);
		flex-shrink: 0;
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

	.help-text {
		font-size: 0.75rem;
		color: var(--md-sys-color-on-surface-variant);
		display: block;
		margin-left: 28px;
		margin-top: 0.25rem;
	}
</style>

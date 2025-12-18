<script lang="ts">
	let {
		id = '',
		name = '',
		label = '',
		value = $bindable(''),
		options = [],
		required = false,
		disabled = false,
		helpText = '',
		errorText = '',
		ariaDescribedBy = '',
		onchange = undefined,
	} = $props();

	let helpId = $derived(id ? `${id}-help` : '');
	let errorId = $derived(id ? `${id}-error` : '');
	let combinedDescribedBy = $derived(
		[ariaDescribedBy, helpText ? helpId : '', errorText ? errorId : ''].filter(Boolean).join(' '),
	);

	function handleChange(e: Event) {
		value = (e.target as HTMLSelectElement).value;
		onchange?.(e);
	}
</script>

<div class="form-field">
	{#if label}
		<label for={id}>
			{label}
			{#if required}
				<span class="required-indicator" aria-label="required">*</span>
			{/if}
		</label>
	{/if}
	<select
		{id}
		{name}
		{value}
		{required}
		{disabled}
		aria-required={required}
		aria-invalid={!!errorText}
		aria-describedby={combinedDescribedBy || undefined}
		onchange={handleChange}
	>
		<option value="">-- Select --</option>
		{#each options as option}
			<option value={option.value}>
				{option.label}
			</option>
		{/each}
	</select>
	{#if helpText}
		<small class="help-text" id={helpId}>{helpText}</small>
	{/if}
	{#if errorText}
		<small class="error-text" id={errorId} role="alert">{errorText}</small>
	{/if}
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
		display: flex;
		gap: 0.25rem;
	}

	.required-indicator {
		color: var(--md-sys-color-error);
	}

	select {
		padding: 0.75rem;
		border: 2px solid var(--md-sys-color-outline);
		border-radius: 4px;
		background-color: var(--md-sys-color-surface);
		color: var(--md-sys-color-on-surface);
		font-size: 0.875rem;
		font-family: inherit;
		cursor: pointer;
		transition: all 0.2s ease;
	}

	select:focus {
		outline: none;
		border-color: var(--md-sys-color-primary);
		box-shadow: 0 0 0 3px color-mix(in srgb, var(--md-sys-color-primary) 12%, transparent);
	}

	select:disabled {
		opacity: 0.38;
		cursor: not-allowed;
	}

	select[aria-invalid='true'] {
		border-color: var(--md-sys-color-error);
	}

	select[aria-invalid='true']:focus {
		box-shadow: 0 0 0 3px color-mix(in srgb, var(--md-sys-color-error) 12%, transparent);
	}

	.help-text {
		font-size: 0.75rem;
		color: var(--md-sys-color-on-surface-variant);
		display: block;
	}

	.error-text {
		font-size: 0.75rem;
		color: var(--md-sys-color-error);
		display: block;
		font-weight: 500;
	}
</style>

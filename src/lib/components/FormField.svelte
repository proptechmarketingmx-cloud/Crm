<script lang="ts">
	let {
		label,
		value = $bindable(),
		type = 'text',
		required = false,
		autocomplete = 'off',
		min,
		max,
		step,
		inputmode,
		pattern,
		maxlength
	}: {
		label: string;
		value: string;
		type?: string;
		required?: boolean;
		autocomplete?: string;
		min?: string;
		max?: string;
		step?: string;
		inputmode?: 'none' | 'text' | 'tel' | 'url' | 'email' | 'numeric' | 'decimal' | 'search';
		pattern?: string;
		maxlength?: number;
	} = $props();
</script>

<label class="grid gap-1.5 text-sm text-slate-300">
	{label}
	<input
		bind:value
		{type}
		{required}
		{autocomplete}
		{min}
		{max}
		{step}
		{inputmode}
		{pattern}
		{maxlength}
		class="rounded-lg border border-white/15 bg-slate-950 px-3 py-2.5 text-white outline-none focus-visible:border-sky-300 focus-visible:ring-2 focus-visible:ring-sky-300/30"
	/>
</label>

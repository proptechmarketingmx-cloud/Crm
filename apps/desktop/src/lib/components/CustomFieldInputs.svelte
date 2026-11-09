<script lang="ts">
  import type { CustomFieldDefinition } from '$lib/api/customFields';

  interface Props {
    definitions: CustomFieldDefinition[];
    values?: Record<string, string>;
    legend?: string;
    disabled?: boolean;
    onchange?: (fieldDefId: string, value: string) => void;
  }

  let {
    definitions,
    values = {},
    legend = 'Custom Fields',
    disabled = false,
    onchange,
  }: Props = $props();

  function valueFor(fieldId: string): string {
    return values[fieldId] ?? '';
  }

  function optionsFor(definition: CustomFieldDefinition): string[] {
    if (!definition.field_options) {
      return [];
    }

    try {
      const parsed = JSON.parse(definition.field_options);
      if (Array.isArray(parsed)) {
        return parsed.filter((item): item is string => typeof item === 'string');
      }
    } catch {
      // Ignore malformed options and render no options.
    }

    return [];
  }
</script>

{#if definitions.length > 0}
  <fieldset class="custom-fields" disabled={disabled}>
    <legend class="custom-fields__legend">{legend}</legend>

    <div class="custom-fields__grid">
      {#each definitions as definition (definition.id)}
        <div class="custom-field">
          <label class="custom-field__label" for={`custom-field-${definition.id}`}>
            {definition.field_name}
          </label>

          {#if definition.field_type === 'select'}
            {@const options = optionsFor(definition)}
            <select
              id={`custom-field-${definition.id}`}
              class="select"
              value={valueFor(definition.id)}
              onchange={(event) => onchange?.(definition.id, (event.target as HTMLSelectElement).value)}
            >
              <option value="">Select...</option>
              {#each options as option (option)}
                <option value={option}>{option}</option>
              {/each}
            </select>

          {:else if definition.field_type === 'boolean'}
            <label class="custom-field__checkbox">
              <input
                id={`custom-field-${definition.id}`}
                type="checkbox"
                checked={valueFor(definition.id) === 'true'}
                onchange={(event) => onchange?.(definition.id, (event.target as HTMLInputElement).checked ? 'true' : '')}
              />
              <span>Enabled</span>
            </label>

          {:else}
            <input
              id={`custom-field-${definition.id}`}
              class="input"
              type={definition.field_type === 'number' ? 'number' : definition.field_type === 'date' ? 'date' : 'text'}
              value={valueFor(definition.id)}
              oninput={(event) => onchange?.(definition.id, (event.target as HTMLInputElement).value)}
            />
          {/if}
        </div>
      {/each}
    </div>
  </fieldset>
{/if}

<style>
  .custom-fields {
    border: var(--border-width) solid var(--border-default);
    border-radius: var(--radius-md);
    padding: var(--space-4);
    background-color: var(--surface-raised);
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  .custom-fields__legend {
    font-size: var(--text-xs);
    font-weight: var(--weight-semibold);
    color: var(--text-secondary);
    letter-spacing: 0.04em;
    text-transform: uppercase;
    padding: 0 var(--space-2);
  }

  .custom-fields__grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--space-4);
  }

  .custom-field {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .custom-field__label {
    font-size: var(--text-xs);
    color: var(--text-secondary);
  }

  .custom-field__checkbox {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
    font-size: var(--text-sm);
    color: var(--text-primary);
    min-height: 34px;
  }

  @media (max-width: 720px) {
    .custom-fields__grid {
      grid-template-columns: 1fr;
    }
  }
</style>

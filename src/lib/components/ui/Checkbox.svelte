<script lang="ts">
  interface Props {
    checked?: boolean;
    disabled?: boolean;
    label?: string;
    id?: string;
    name?: string;
    onchange?: (checked: boolean) => void;
  }

  let {
    checked = $bindable(false),
    disabled = false,
    label = '',
    id = '',
    name = '',
    onchange
  }: Props = $props();

  function handleChange(event: Event) {
    const target = event.target as HTMLInputElement;
    const newChecked = target.checked;
    checked = newChecked;
    onchange?.(newChecked);
  }
</script>

<div class="checkbox-wrapper">
  <input
    type="checkbox"
    class="checkbox-input"
    {id}
    {name}
    {disabled}
    bind:checked
    onchange={handleChange}
  />
  <label for={id} class="checkbox-label">
    {label}
  </label>
</div>

<style>
  .checkbox-wrapper {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .checkbox-input {
    margin: 0;
  }

  .checkbox-label {
    font-size: 0.7rem;
    color: var(--text);
    cursor: pointer;
    max-width: 100px;
    word-wrap: break-word;
    white-space: normal;
    line-height: 1.2;
  }
</style>

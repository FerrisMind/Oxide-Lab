<script lang="ts">
  export let checked = false;
  export let disabled = false;
  export let label = '';
  export let id = '';
  export let name = '';

  function handleChange(event: Event) {
    const target = event.target as HTMLInputElement;
    checked = target.checked;
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
    on:change={handleChange}
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

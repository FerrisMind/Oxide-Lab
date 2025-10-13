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
    display: inline-block;
  }
</style>

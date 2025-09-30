<script lang="ts">
  
  // Типы для пресетов
  export interface GenerationParams {
    temperature: number;
    temperature_enabled: boolean;
    top_k_enabled: boolean;
    top_k_value: number;
    top_p_enabled: boolean;
    top_p_value: number;
    min_p_enabled: boolean;
    min_p_value: number;
    repeat_penalty_enabled: boolean;
    repeat_penalty_value: number;
    ctx_limit_value: number;
  }
  
  export interface Preset {
    id: string;
    name: string;
    description: string;
    params: GenerationParams;
    isBuiltin: boolean;
  }
  
  // Встроенные пресеты (соответствуют Rust SamplingOptions)
  const builtinPresets: Preset[] = [
    {
      id: 'balanced',
      name: 'Сбалансированный',
      description: 'Оптимальный баланс качества и разнообразия для большинства задач',
      isBuiltin: true,
      params: {
        temperature: 0.7,
        temperature_enabled: true,
        top_k_enabled: true,
        top_k_value: 20,
        top_p_enabled: true,
        top_p_value: 0.9,
        min_p_enabled: true,
        min_p_value: 0.0,
        repeat_penalty_enabled: true,
        repeat_penalty_value: 1.1,
        ctx_limit_value: 4096,
      },
    },
    {
      id: 'precise',
      name: 'Точный',
      description: 'Консервативные настройки для более детерминированного вывода',
      isBuiltin: true,
      params: {
        temperature: 0.2,
        temperature_enabled: true,
        top_k_enabled: true,
        top_k_value: 10,
        top_p_enabled: true,
        top_p_value: 0.8,
        min_p_enabled: true,
        min_p_value: 0.0,
        repeat_penalty_enabled: true,
        repeat_penalty_value: 1.2,
        ctx_limit_value: 4096,
      },
    },
    {
      id: 'creative',
      name: 'Креативный',
      description: 'Настройки для более разнообразного и творческого вывода',
      isBuiltin: true,
      params: {
        temperature: 0.9,
        temperature_enabled: true,
        top_k_enabled: true,
        top_k_value: 50,
        top_p_enabled: true,
        top_p_value: 0.95,
        min_p_enabled: true,
        min_p_value: 0.0,
        repeat_penalty_enabled: true,
        repeat_penalty_value: 1.05,
        ctx_limit_value: 4096,
      },
    },
    {
      id: 'deterministic',
      name: 'Детерминированный',
      description: 'Максимально предсказуемый вывод (argmax)',
      isBuiltin: true,
      params: {
        temperature: 0.0,
        temperature_enabled: false,
        top_k_enabled: false,
        top_k_value: 0,
        top_p_enabled: false,
        top_p_value: 0,
        min_p_enabled: false,
        min_p_value: 0.0,
        repeat_penalty_enabled: true,
        repeat_penalty_value: 1.1,
        ctx_limit_value: 4096,
      },
    },
  ];
  
  // Props
  let {
    currentParams = $bindable(),
    onApplyPreset = (_params: GenerationParams) => {},
    class: className = ''
  }: {
    currentParams?: GenerationParams;
    onApplyPreset?: (params: GenerationParams) => void;
    class?: string;
  } = $props();
  
  // Локальное состояние
  let customPresets = $state<Preset[]>([]);
  let selectedPresetId = $state<string | null>(null);
  let showSaveDialog = $state(false);
  let newPresetName = $state('');
  let newPresetDescription = $state('');
  
  // Все пресеты
  let allPresets = $derived([...builtinPresets, ...customPresets]);
  
  // Применить пресет
  function applyPreset(preset: Preset) {
    selectedPresetId = preset.id;
    if (currentParams) {
      Object.assign(currentParams, preset.params);
    }
    onApplyPreset(preset.params);
  }
  
  // Сохранить текущие параметры как пресет
  function saveAsPreset() {
    if (!newPresetName.trim() || !currentParams) return;
    
    const newPreset: Preset = {
      id: `custom-${Date.now()}`,
      name: newPresetName.trim(),
      description: newPresetDescription.trim(),
      isBuiltin: false,
      params: { ...currentParams },
    };
    
    customPresets.push(newPreset);
    
    // Сохраняем в localStorage
    localStorage.setItem('oxide-lab-presets', JSON.stringify(customPresets));
    
    // Сбрасываем диалог
    showSaveDialog = false;
    newPresetName = '';
    newPresetDescription = '';
  }
  
  // Удалить пользовательский пресет
  function deletePreset(id: string) {
    customPresets = customPresets.filter(p => p.id !== id);
    localStorage.setItem('oxide-lab-presets', JSON.stringify(customPresets));
    
    if (selectedPresetId === id) {
      selectedPresetId = null;
    }
  }
  
  // Загрузить пресеты из localStorage при монтировании
  $effect(() => {
    const saved = localStorage.getItem('oxide-lab-presets');
    if (saved) {
      try {
        customPresets = JSON.parse(saved);
      } catch (error) {
        console.error('Failed to load custom presets:', error);
      }
    }
  });
</script>

<div class="param-presets {className}">
  <div class="presets-header">
    <h3>Пресеты параметров</h3>
    <button
      class="btn btn-save"
      onclick={() => (showSaveDialog = true)}
      disabled={!currentParams}
      title="Сохранить текущие параметры как пресет"
    >
      + Сохранить
    </button>
  </div>
  
  <!-- Список пресетов -->
  <div class="presets-list">
    {#each allPresets as preset (preset.id)}
      <div
        class="preset-card"
        class:selected={selectedPresetId === preset.id}
        class:builtin={preset.isBuiltin}
      >
        <div class="preset-info">
          <div class="preset-name">
            {preset.name}
            {#if preset.isBuiltin}
              <span class="badge badge-builtin">Встроенный</span>
            {/if}
          </div>
          <div class="preset-description">
            {preset.description}
          </div>
          
          <!-- Краткая информация о параметрах -->
          <div class="preset-params-summary">
            <span class="param-chip">
              T: {preset.params.temperature.toFixed(2)}
            </span>
            {#if preset.params.top_k_enabled}
              <span class="param-chip">
                top-k: {preset.params.top_k_value}
              </span>
            {/if}
            {#if preset.params.top_p_enabled}
              <span class="param-chip">
                top-p: {preset.params.top_p_value.toFixed(2)}
              </span>
            {/if}
            {#if preset.params.repeat_penalty_enabled}
              <span class="param-chip">
                RP: {preset.params.repeat_penalty_value.toFixed(2)}
              </span>
            {/if}
          </div>
        </div>
        
        <div class="preset-actions">
          <button
            class="btn btn-apply"
            onclick={() => applyPreset(preset)}
          >
            Применить
          </button>
          
          {#if !preset.isBuiltin}
            <button
              class="btn btn-delete"
              onclick={() => deletePreset(preset.id)}
              title="Удалить пресет"
            >
              🗑️
            </button>
          {/if}
        </div>
      </div>
    {/each}
    
    {#if allPresets.length === builtinPresets.length}
      <div class="empty-state">
        <p>У вас пока нет пользовательских пресетов</p>
        <p class="hint">Настройте параметры и сохраните их как пресет</p>
      </div>
    {/if}
  </div>
  
  <!-- Диалог сохранения пресета -->
  {#if showSaveDialog}
    <div 
      class="dialog-overlay" 
      onclick={() => (showSaveDialog = false)}
      onkeydown={(e) => e.key === 'Escape' && (showSaveDialog = false)}
      role="button"
      tabindex="0"
      aria-label="Закрыть диалог"
    >
      <div 
        class="dialog" 
        onclick={(e) => e.stopPropagation()}
        onkeydown={(e) => e.stopPropagation()}
        role="dialog"
        aria-modal="true"
        tabindex="0"
      >
        <h4>Сохранить пресет</h4>
        
        <div class="form-group">
          <label for="preset-name">Название</label>
          <input
            id="preset-name"
            type="text"
            bind:value={newPresetName}
            placeholder="Мой пресет"
            maxlength="50"
          />
        </div>
        
        <div class="form-group">
          <label for="preset-description">Описание</label>
          <textarea
            id="preset-description"
            bind:value={newPresetDescription}
            placeholder="Краткое описание пресета..."
            maxlength="200"
            rows="3"
          ></textarea>
        </div>
        
        <div class="dialog-actions">
          <button
            class="btn btn-secondary"
            onclick={() => (showSaveDialog = false)}
          >
            Отмена
          </button>
          <button
            class="btn btn-primary"
            onclick={saveAsPreset}
            disabled={!newPresetName.trim()}
          >
            Сохранить
          </button>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .param-presets {
    background: var(--card);
    border: 1px solid var(--border-color);
    border-radius: 8px;
    padding: 1rem;
  }
  
  .presets-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
  }
  
  .presets-header h3 {
    margin: 0;
    font-size: 1.125rem;
    font-weight: 600;
    color: var(--text);
  }
  
  .btn-save {
    padding: 0.5rem 0.75rem;
    background: var(--accent, #3498db);
    color: white;
    border: none;
    border-radius: 4px;
    font-size: 0.75rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
  }
  
  .btn-save:hover:not(:disabled) {
    background: var(--accent-hover, #2980b9);
  }
  
  .btn-save:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  
  .presets-list {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    max-height: 500px;
    overflow-y: auto;
  }
  
  .preset-card {
    padding: 0.75rem;
    background: rgba(149, 165, 166, 0.05);
    border: 2px solid transparent;
    border-radius: 6px;
    transition: all 0.2s ease;
  }
  
  .preset-card.selected {
    border-color: var(--accent, #3498db);
    background: rgba(52, 152, 219, 0.1);
  }
  
  .preset-card.builtin {
    background: rgba(46, 204, 113, 0.05);
  }
  
  .preset-info {
    margin-bottom: 0.75rem;
  }
  
  .preset-name {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-weight: 600;
    font-size: 0.875rem;
    color: var(--text);
    margin-bottom: 0.25rem;
  }
  
  .badge {
    padding: 0.125rem 0.5rem;
    border-radius: 12px;
    font-size: 0.625rem;
    font-weight: 500;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }
  
  .badge-builtin {
    background: var(--success, #2ecc71);
    color: white;
  }
  
  .preset-description {
    font-size: 0.75rem;
    color: var(--muted);
    line-height: 1.4;
    margin-bottom: 0.5rem;
  }
  
  .preset-params-summary {
    display: flex;
    flex-wrap: wrap;
    gap: 0.375rem;
  }
  
  .param-chip {
    padding: 0.25rem 0.5rem;
    background: rgba(52, 152, 219, 0.1);
    border: 1px solid rgba(52, 152, 219, 0.2);
    border-radius: 4px;
    font-size: 0.625rem;
    font-family: monospace;
    color: var(--info, #3498db);
  }
  
  .preset-actions {
    display: flex;
    gap: 0.5rem;
  }
  
  .btn {
    padding: 0.5rem 0.75rem;
    border: none;
    border-radius: 4px;
    font-size: 0.75rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
  }
  
  .btn-apply {
    flex: 1;
    background: var(--accent, #3498db);
    color: white;
  }
  
  .btn-apply:hover {
    background: var(--accent-hover, #2980b9);
  }
  
  .btn-delete {
    padding: 0.5rem 0.625rem;
    background: transparent;
    border: 1px solid var(--border-color);
  }
  
  .btn-delete:hover {
    background: var(--error, #e74c3c);
    border-color: var(--error, #e74c3c);
  }
  
  .empty-state {
    padding: 2rem 1rem;
    text-align: center;
    color: var(--muted);
  }
  
  .empty-state p {
    margin: 0.5rem 0;
    font-size: 0.875rem;
  }
  
  .hint {
    font-size: 0.75rem;
    color: var(--muted);
    opacity: 0.8;
  }
  
  /* Диалог */
  .dialog-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }
  
  .dialog {
    background: var(--card);
    border: 1px solid var(--border-color);
    border-radius: 8px;
    padding: 1.5rem;
    max-width: 500px;
    width: 90%;
  }
  
  .dialog h4 {
    margin: 0 0 1rem 0;
    font-size: 1.25rem;
    font-weight: 600;
    color: var(--text);
  }
  
  .form-group {
    margin-bottom: 1rem;
  }
  
  .form-group label {
    display: block;
    margin-bottom: 0.5rem;
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--text);
  }
  
  .form-group input,
  .form-group textarea {
    width: 100%;
    padding: 0.5rem;
    background: var(--bg);
    border: 1px solid var(--border-color);
    border-radius: 4px;
    font-size: 0.875rem;
    color: var(--text);
    font-family: inherit;
  }
  
  .form-group textarea {
    resize: vertical;
  }
  
  .dialog-actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.75rem;
    margin-top: 1.5rem;
  }
  
  .btn-secondary {
    background: transparent;
    border: 1px solid var(--border-color);
    color: var(--text);
  }
  
  .btn-secondary:hover {
    background: rgba(149, 165, 166, 0.1);
  }
  
  .btn-primary {
    background: var(--accent, #3498db);
    color: white;
  }
  
  .btn-primary:hover:not(:disabled) {
    background: var(--accent-hover, #2980b9);
  }
  
  .btn-primary:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  
  /* Скроллбар */
  .presets-list::-webkit-scrollbar {
    width: 6px;
  }
  
  .presets-list::-webkit-scrollbar-track {
    background: transparent;
  }
  
  .presets-list::-webkit-scrollbar-thumb {
    background: rgba(179, 205, 224, 0.6);
    border-radius: 3px;
  }
  
  .presets-list::-webkit-scrollbar-thumb:hover {
    background: rgba(179, 205, 224, 0.8);
  }
</style>

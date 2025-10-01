<script lang="ts">
  /**
   * Reusable confirmation dialog component
   * Provides accessible modal dialog with focus trap
   */

  let {
    open = $bindable(false),
    title = 'Подтверждение',
    message = 'Вы уверены?',
    confirmText = 'Подтвердить',
    cancelText = 'Отмена',
    danger = false,
    onConfirm = () => {},
    onCancel = () => {},
  }: {
    open: boolean;
    title?: string;
    message?: string;
    confirmText?: string;
    cancelText?: string;
    danger?: boolean;
    onConfirm?: () => void;
    onCancel?: () => void;
  } = $props();

  let dialogElement = $state<HTMLDivElement>();
  let confirmButton = $state<HTMLButtonElement>();
  let cancelButton = $state<HTMLButtonElement>();

  // Handle confirm
  function handleConfirm() {
    onConfirm();
    open = false;
  }

  // Handle cancel
  function handleCancel() {
    onCancel();
    open = false;
  }

  // Handle Escape key
  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      event.preventDefault();
      handleCancel();
    }
  }

  // Handle backdrop click
  function handleBackdropClick(event: MouseEvent) {
    if (event.target === dialogElement) {
      handleCancel();
    }
  }

  // Trap focus within dialog
  function trapFocus(event: KeyboardEvent) {
    if (event.key !== 'Tab') return;

    const focusableElements = [confirmButton, cancelButton].filter(
      (el): el is HTMLButtonElement => !!el,
    );
    if (focusableElements.length === 0) return;

    const firstElement = focusableElements[0];
    const lastElement = focusableElements[focusableElements.length - 1];

    if (event.shiftKey) {
      if (document.activeElement === firstElement) {
        event.preventDefault();
        lastElement?.focus();
      }
    } else {
      if (document.activeElement === lastElement) {
        event.preventDefault();
        firstElement?.focus();
      }
    }
  }

  // Focus first button when dialog opens
  $effect(() => {
    if (open && cancelButton) {
      // Focus cancel button by default for safety
      cancelButton.focus();
    }
  });
</script>

{#if open}
  <div
    bind:this={dialogElement}
    class="dialog-backdrop"
    role="dialog"
    aria-modal="true"
    aria-labelledby="dialog-title"
    aria-describedby="dialog-description"
    tabindex="-1"
    onkeydown={handleKeydown}
    onclick={handleBackdropClick}
  >
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="dialog-content" onkeydown={trapFocus}>
      <div class="dialog-header">
        <h2 id="dialog-title" class="dialog-title">{title}</h2>
      </div>

      <div class="dialog-body">
        <p id="dialog-description" class="dialog-message">{message}</p>
      </div>

      <div class="dialog-footer">
        <button
          bind:this={cancelButton}
          type="button"
          class="btn btn-secondary"
          onclick={handleCancel}
        >
          {cancelText}
        </button>

        <button
          bind:this={confirmButton}
          type="button"
          class="btn"
          class:btn-danger={danger}
          class:btn-primary={!danger}
          onclick={handleConfirm}
        >
          {confirmText}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .dialog-backdrop {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    padding: 1rem;
    animation: fadeIn 0.2s ease;
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }

  .dialog-content {
    background: var(--card);
    border: 1px solid var(--border-color);
    border-radius: 8px;
    box-shadow: 0 10px 40px rgba(0, 0, 0, 0.2);
    max-width: 500px;
    width: 100%;
    animation: slideIn 0.2s ease;
  }

  @keyframes slideIn {
    from {
      transform: translateY(-20px);
      opacity: 0;
    }
    to {
      transform: translateY(0);
      opacity: 1;
    }
  }

  .dialog-header {
    padding: 1.5rem 1.5rem 1rem;
    border-bottom: 1px solid var(--border-color);
  }

  .dialog-title {
    margin: 0;
    font-size: 1.25rem;
    font-weight: 600;
    color: var(--text);
  }

  .dialog-body {
    padding: 1.5rem;
  }

  .dialog-message {
    margin: 0;
    font-size: 1rem;
    color: var(--text);
    line-height: 1.5;
  }

  .dialog-footer {
    padding: 1rem 1.5rem 1.5rem;
    display: flex;
    justify-content: flex-end;
    gap: 0.75rem;
  }

  .btn {
    padding: 0.625rem 1.25rem;
    border: none;
    border-radius: 6px;
    font-size: 0.9375rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .btn:focus {
    outline: 2px solid var(--accent, #3498db);
    outline-offset: 2px;
  }

  .btn-primary {
    background: var(--accent, #3498db);
    color: white;
  }

  .btn-primary:hover {
    background: var(--accent-hover, #2980b9);
  }

  .btn-secondary {
    background: rgba(149, 165, 166, 0.1);
    color: var(--text);
  }

  .btn-secondary:hover {
    background: rgba(149, 165, 166, 0.2);
  }

  .btn-danger {
    background: var(--error, #e74c3c);
    color: white;
  }

  .btn-danger:hover {
    background: #c0392b;
  }

  /* Адаптивность */
  @media (max-width: 768px) {
    .dialog-content {
      max-width: 100%;
    }

    .dialog-header {
      padding: 1.25rem 1.25rem 0.875rem;
    }

    .dialog-title {
      font-size: 1.125rem;
    }

    .dialog-body {
      padding: 1.25rem;
    }

    .dialog-footer {
      padding: 0.875rem 1.25rem 1.25rem;
      flex-direction: column-reverse;
    }

    .btn {
      width: 100%;
    }
  }
</style>

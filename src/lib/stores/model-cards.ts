import { derived, get, writable } from 'svelte/store';
import type { ModelCardSummary } from '$lib/types/model-cards';
import { ModelCardsService } from '$lib/services/model-cards';

type FilterState = {
  searchText: string;
  family: string;
  format: 'gguf' | 'safetensors' | '';
};

export const modelCards = writable<ModelCardSummary[]>([]);
export const modelCardsLoading = writable(false);
export const modelCardsError = writable<string | null>(null);
export const modelCardFilters = writable<FilterState>({
  searchText: '',
  family: '',
  format: '',
});
export const modelCardsVersion = writable<number | null>(null);
export const modelCardsStatus = writable<string | null>(null);

export const filteredModelCards = derived([modelCards, modelCardFilters], ([$cards, $filters]) => {
  const query = $filters.searchText.trim().toLowerCase();
  return $cards.filter((card) => {
    if ($filters.family && card.family !== $filters.family) {
      return false;
    }
    if ($filters.format) {
      const hasFormat = card.supported_formats.some(
        (format) => format.toLowerCase() === $filters.format,
      );
      if (!hasFormat) {
        return false;
      }
    }

    if (query) {
      const haystack = [card.name, card.description, card.hf_repo_id, card.tags.join(' ')]
        .join(' ')
        .toLowerCase();
      if (!haystack.includes(query)) {
        return false;
      }
    }
    return true;
  });
});

export const uniqueFamilies = derived(modelCards, ($cards) => {
  return Array.from(
    new Set($cards.map((card) => card.family ?? '').filter((item) => item.length)),
  ).sort((a, b) => a.localeCompare(b));
});

export async function loadModelCards(force: boolean = false): Promise<void> {
  if (!force && get(modelCards).length) {
    return;
  }

  modelCardsLoading.set(true);
  modelCardsError.set(null);

  try {
    const response = await ModelCardsService.getModelCards();
    modelCards.set(response.cards);
    modelCardsVersion.set(response.version);
  } catch (error) {
    modelCardsError.set(error instanceof Error ? error.message : String(error));
  } finally {
    modelCardsLoading.set(false);
  }
}

export async function importModelCards(path: string): Promise<void> {
  modelCardsLoading.set(true);
  modelCardsStatus.set(null);

  try {
    const response = await ModelCardsService.importModelCards(path);
    modelCards.set(response.cards);
    modelCardsVersion.set(response.version);
    modelCardsStatus.set(`Импортован конфиг версии ${response.version}`);
  } catch (error) {
    modelCardsStatus.set(
      `Ошибка импорта: ${error instanceof Error ? error.message : String(error)}`,
    );
  } finally {
    modelCardsLoading.set(false);
  }
}

export async function resetModelCards(): Promise<void> {
  modelCardsLoading.set(true);
  modelCardsStatus.set(null);

  try {
    const response = await ModelCardsService.resetModelCards();
    modelCards.set(response.cards);
    modelCardsVersion.set(response.version);
    modelCardsStatus.set(`Сброшено до версии ${response.version}`);
  } catch (error) {
    modelCardsStatus.set(
      `Ошибка сброса: ${error instanceof Error ? error.message : String(error)}`,
    );
  } finally {
    modelCardsLoading.set(false);
  }
}

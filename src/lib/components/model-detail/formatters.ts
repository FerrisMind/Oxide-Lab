/**
 * Utility functions for formatting model data
 */

/**
 * Format number with K/M suffixes
 */
export function formatNumber(num: number): string {
  if (num >= 1000000) {
    return `${(num / 1000000).toFixed(1)}M`;
  } else if (num >= 1000) {
    return `${(num / 1000).toFixed(1)}K`;
  }
  return num.toString();
}

/**
 * Format date to Russian locale
 */
export function formatDate(dateString: string): string {
  const date = new Date(dateString);
  return date.toLocaleDateString('ru-RU', {
    year: 'numeric',
    month: 'long',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  });
}

/**
 * Get color for model tag
 */
export function getTagColor(tag: string): string {
  const tagColors: Record<string, string> = {
    gguf: '#10b981',
    safetensors: '#3b82f6',
    llama: '#8b5cf6',
    mistral: '#f59e0b',
    gemma: '#ef4444',
    qwen: '#06b6d4',
    pytorch: '#ee4b2b',
    transformers: '#ff6b6b',
    'text-generation': '#4ecdc4',
    conversational: '#45b7d1',
  };
  return tagColors[tag.toLowerCase()] || '#6b7280';
}

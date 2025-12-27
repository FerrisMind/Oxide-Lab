/**
 * Model Icons Utility
 * 
 * Maps model families to icon names from @lobehub/icons-static-svg
 */

const FAMILY_ICON_MAP: Record<string, string> = {
  // Major providers
  'llama': 'meta',
  'llama2': 'meta',
  'llama3': 'meta',
  'meta-llama': 'meta',
  'codellama': 'meta',
  
  // Mistral family
  'mistral': 'mistral-color',
  'mixtral': 'mistral-color',
  
  // Qwen family
  'qwen': 'qwen-color',
  'qwen2': 'qwen-color',
  'qwen2.5': 'qwen-color',
  
  // Google
  'gemma': 'gemma-color',
  'gemma2': 'gemma-color',
  
  // Microsoft
  'phi': 'phi-color',
  'phi2': 'phi-color',
  'phi3': 'phi-color',
  
  // DeepSeek
  'deepseek': 'deepseek-color',
  'deepseek-coder': 'deepseek-color',
  'deepseek-v2': 'deepseek-color',
  
  // Anthropic
  'claude': 'anthropic',
  
  // OpenAI
  'gpt': 'openai',
  'chatgpt': 'openai',
  
  // Stability
  'stable': 'stability',
  'stablelm': 'stability',
  
  // Yi
  'yi': 'yi-color',
  
  // Cohere
  'command': 'cohere-color',
  'command-r': 'cohere-color',
  
  // Others
  'falcon': 'falcon',
  'vicuna': 'vicuna',
  'wizardlm': 'wizardlm',
  'starcoder': 'huggingface',
  'codestral': 'mistral-color',
};

/**
 * Get the icon name for a model family
 */
export function getModelIconName(family: string | null | undefined): string {
  if (!family) return 'huggingface';
  
  const normalized = family.toLowerCase().trim();
  
  // Direct match
  if (FAMILY_ICON_MAP[normalized]) {
    return FAMILY_ICON_MAP[normalized];
  }
  
  // Partial match
  for (const [key, icon] of Object.entries(FAMILY_ICON_MAP)) {
    if (normalized.includes(key) || key.includes(normalized)) {
      return icon;
    }
  }
  
  // Default
  return 'huggingface';
}

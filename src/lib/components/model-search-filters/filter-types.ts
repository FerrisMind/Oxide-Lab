// Types and utilities for ModelSearchFilters components

export interface FilterOption {
  id: string;
  label: string;
  color?: string;
}

export interface FilterState {
  selectedFormats: string[];
  selectedPipelineTags: string[];
  selectedLibraries: string[];
  selectedLanguages: string[];
  selectedLicenses: string[];
  authorFilter: string;
  searchQuery: string;
}

export interface SearchEvent {
  query: string;
  formats: string[];
  pipelineTags: string[];
  libraries: string[];
  languages: string[];
  licenses: string[];
  author: string;
}

// Import filter configurations
import { filterConfig } from './filter-config';

// Export filter options from config
export const availableFormats = filterConfig.formats;
export const availablePipelineTags = filterConfig.pipelineTags;
export const availableLibraries = filterConfig.libraries;
export const availableLanguages = filterConfig.languages;
export const availableLicenses = filterConfig.licenses;

// Utility functions
export function toggleArrayItem<T>(array: T[], item: T): T[] {
  if (array.includes(item)) {
    return array.filter(i => i !== item);
  } else {
    return [...array, item];
  }
}

export function hasActiveFilters(state: FilterState): boolean {
  return (
    state.selectedFormats.length > 0 ||
    state.selectedPipelineTags.length > 0 ||
    state.selectedLibraries.length > 0 ||
    state.selectedLanguages.length > 0 ||
    state.selectedLicenses.length > 0 ||
    !!state.searchQuery ||
    !!state.authorFilter
  );
}

export function clearAllFilters(): FilterState {
  return {
    selectedFormats: [],
    selectedPipelineTags: [],
    selectedLibraries: [],
    selectedLanguages: [],
    selectedLicenses: [],
    authorFilter: '',
    searchQuery: ''
  };
}
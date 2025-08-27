import type { FilterState, SearchEvent } from './filter-types';

// Utility functions for filter operations
export function createSearchEvent(state: {
  searchQuery: string;
  selectedFormats: string[];
  selectedPipelineTags: string[];
  selectedLibraries: string[];
  selectedLanguages: string[];
  selectedLicenses: string[];
  authorFilter: string;
}): SearchEvent {
  return {
    query: state.searchQuery,
    formats: state.selectedFormats,
    pipelineTags: state.selectedPipelineTags,
    libraries: state.selectedLibraries,
    languages: state.selectedLanguages,
    licenses: state.selectedLicenses,
    author: state.authorFilter,
  };
}

export function toggleFormat(selectedFormats: string[], formatId: string): string[] {
  if (selectedFormats.includes(formatId)) {
    return selectedFormats.filter((f) => f !== formatId);
  } else {
    return [...selectedFormats, formatId];
  }
}

export function togglePipelineTag(selectedPipelineTags: string[], tagId: string): string[] {
  if (selectedPipelineTags.includes(tagId)) {
    return selectedPipelineTags.filter((t) => t !== tagId);
  } else {
    return [...selectedPipelineTags, tagId];
  }
}

export function toggleLibrary(selectedLibraries: string[], libraryId: string): string[] {
  if (selectedLibraries.includes(libraryId)) {
    return selectedLibraries.filter((l) => l !== libraryId);
  } else {
    return [...selectedLibraries, libraryId];
  }
}

export function toggleLanguage(selectedLanguages: string[], languageId: string): string[] {
  if (selectedLanguages.includes(languageId)) {
    return selectedLanguages.filter((l) => l !== languageId);
  } else {
    return [...selectedLanguages, languageId];
  }
}

export function toggleLicense(selectedLicenses: string[], licenseId: string): string[] {
  if (selectedLicenses.includes(licenseId)) {
    return selectedLicenses.filter((l) => l !== licenseId);
  } else {
    return [...selectedLicenses, licenseId];
  }
}

export function clearFilters(): FilterState {
  return {
    selectedFormats: [],
    selectedPipelineTags: [],
    selectedLibraries: [],
    selectedLanguages: [],
    selectedLicenses: [],
    authorFilter: '',
    searchQuery: '',
  };
}

export function removeFormat(selectedFormats: string[], formatId: string): string[] {
  return selectedFormats.filter((f) => f !== formatId);
}

export function removePipelineTag(selectedPipelineTags: string[], tagId: string): string[] {
  return selectedPipelineTags.filter((t) => t !== tagId);
}

export function removeLibrary(selectedLibraries: string[], libraryId: string): string[] {
  return selectedLibraries.filter((l) => l !== libraryId);
}

export function removeLanguage(selectedLanguages: string[], languageId: string): string[] {
  return selectedLanguages.filter((l) => l !== languageId);
}

export function removeLicense(selectedLicenses: string[], licenseId: string): string[] {
  return selectedLicenses.filter((l) => l !== licenseId);
}

// Generic filter removal function
export function removeFilter(
  filterType: 'format' | 'pipelineTag' | 'library' | 'language' | 'license',
  filterId: string,
  currentState: {
    selectedFormats: string[];
    selectedPipelineTags: string[];
    selectedLibraries: string[];
    selectedLanguages: string[];
    selectedLicenses: string[];
  },
): {
  selectedFormats: string[];
  selectedPipelineTags: string[];
  selectedLibraries: string[];
  selectedLanguages: string[];
  selectedLicenses: string[];
} {
  switch (filterType) {
    case 'format':
      return {
        ...currentState,
        selectedFormats: removeFormat(currentState.selectedFormats, filterId),
      };
    case 'pipelineTag':
      return {
        ...currentState,
        selectedPipelineTags: removePipelineTag(currentState.selectedPipelineTags, filterId),
      };
    case 'library':
      return {
        ...currentState,
        selectedLibraries: removeLibrary(currentState.selectedLibraries, filterId),
      };
    case 'language':
      return {
        ...currentState,
        selectedLanguages: removeLanguage(currentState.selectedLanguages, filterId),
      };
    case 'license':
      return {
        ...currentState,
        selectedLicenses: removeLicense(currentState.selectedLicenses, filterId),
      };
    default:
      return currentState;
  }
}

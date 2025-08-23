import { createEventDispatcher } from 'svelte';
import type { SearchEvent } from './filter-types';
import { 
  createSearchEvent,
  toggleFormat,
  togglePipelineTag,
  toggleLibrary,
  toggleLanguage,
  toggleLicense,
  clearFilters,
  removeFormat,
  removePipelineTag,
  removeLibrary,
  removeLanguage,
  removeLicense
} from './filter-utils';

// This function creates event handlers for the ModelSearchFilters component
export function createEventHandlers(
  dispatch: ReturnType<typeof createEventDispatcher>,
  state: {
    searchQuery: string;
    selectedFormats: string[];
    selectedPipelineTags: string[];
    selectedLibraries: string[];
    selectedLanguages: string[];
    selectedLicenses: string[];
    authorFilter: string;
  },
  setters: {
    setSearchQuery: (value: string) => void;
    setSelectedFormats: (value: string[]) => void;
    setSelectedPipelineTags: (value: string[]) => void;
    setSelectedLibraries: (value: string[]) => void;
    setSelectedLanguages: (value: string[]) => void;
    setSelectedLicenses: (value: string[]) => void;
    setAuthorFilter: (value: string) => void;
  }
) {
  function handleSearch() {
    dispatch('search', createSearchEvent({
      searchQuery: state.searchQuery,
      selectedFormats: state.selectedFormats,
      selectedPipelineTags: state.selectedPipelineTags,
      selectedLibraries: state.selectedLibraries,
      selectedLanguages: state.selectedLanguages,
      selectedLicenses: state.selectedLicenses,
      authorFilter: state.authorFilter
    }));
  }

  function handleToggleFormat(event: CustomEvent<string>) {
    const formatId = event.detail;
    const newFormats = toggleFormat(state.selectedFormats, formatId);
    setters.setSelectedFormats(newFormats);
    handleSearch();
  }

  function handleTogglePipelineTag(event: CustomEvent<string>) {
    const tagId = event.detail;
    const newTags = togglePipelineTag(state.selectedPipelineTags, tagId);
    setters.setSelectedPipelineTags(newTags);
    handleSearch();
  }

  function handleToggleLibrary(event: CustomEvent<string>) {
    const libraryId = event.detail;
    const newLibraries = toggleLibrary(state.selectedLibraries, libraryId);
    setters.setSelectedLibraries(newLibraries);
    handleSearch();
  }

  function handleToggleLanguage(event: CustomEvent<string>) {
    const languageId = event.detail;
    const newLanguages = toggleLanguage(state.selectedLanguages, languageId);
    setters.setSelectedLanguages(newLanguages);
    handleSearch();
  }

  function handleToggleLicense(event: CustomEvent<string>) {
    const licenseId = event.detail;
    const newLicenses = toggleLicense(state.selectedLicenses, licenseId);
    setters.setSelectedLicenses(newLicenses);
    handleSearch();
  }

  function handleClearFilters() {
    const clearedState = clearFilters();
    setters.setSelectedFormats(clearedState.selectedFormats);
    setters.setSelectedPipelineTags(clearedState.selectedPipelineTags);
    setters.setSelectedLibraries(clearedState.selectedLibraries);
    setters.setSelectedLanguages(clearedState.selectedLanguages);
    setters.setSelectedLicenses(clearedState.selectedLicenses);
    setters.setSearchQuery(clearedState.searchQuery);
    setters.setAuthorFilter(clearedState.authorFilter);
    handleSearch();
  }

  function handleRemoveSearchQuery() {
    setters.setSearchQuery('');
    handleSearch();
  }

  function handleRemoveAuthorFilter() {
    setters.setAuthorFilter('');
    handleSearch();
  }

  function handleRemoveFormat(event: CustomEvent<string>) {
    const formatId = event.detail;
    const newFormats = removeFormat(state.selectedFormats, formatId);
    setters.setSelectedFormats(newFormats);
    handleSearch();
  }

  function handleRemovePipelineTag(event: CustomEvent<string>) {
    const tagId = event.detail;
    const newTags = removePipelineTag(state.selectedPipelineTags, tagId);
    setters.setSelectedPipelineTags(newTags);
    handleSearch();
  }

  function handleRemoveLibrary(event: CustomEvent<string>) {
    const libraryId = event.detail;
    const newLibraries = removeLibrary(state.selectedLibraries, libraryId);
    setters.setSelectedLibraries(newLibraries);
    handleSearch();
  }

  function handleRemoveLanguage(event: CustomEvent<string>) {
    const languageId = event.detail;
    const newLanguages = removeLanguage(state.selectedLanguages, languageId);
    setters.setSelectedLanguages(newLanguages);
    handleSearch();
  }

  function handleRemoveLicense(event: CustomEvent<string>) {
    const licenseId = event.detail;
    const newLicenses = removeLicense(state.selectedLicenses, licenseId);
    setters.setSelectedLicenses(newLicenses);
    handleSearch();
  }

  return {
    handleSearch,
    handleToggleFormat,
    handleTogglePipelineTag,
    handleToggleLibrary,
    handleToggleLanguage,
    handleToggleLicense,
    handleClearFilters,
    handleRemoveSearchQuery,
    handleRemoveAuthorFilter,
    handleRemoveFormat,
    handleRemovePipelineTag,
    handleRemoveLibrary,
    handleRemoveLanguage,
    handleRemoveLicense
  };
}
/**
 * Backend Integration Module
 *
 * Centralizes initialization and cleanup of all Tauri backend connections.
 * This module follows the Single Responsibility Principle - it only manages
 * backend connection lifecycle.
 */

import { ensureDownloadManager, stopDownloadManager } from '$lib/stores/download-manager';
import { loadModelCards } from '$lib/stores/model-cards';
import { performanceService } from '$lib/services/performance-service';

// Track initialization state
let isInitialized = false;

/**
 * Initialize all backend connections.
 * Should be called once when the application starts (e.g., in +layout.svelte onMount).
 */
export async function initializeBackend(): Promise<void> {
    if (isInitialized) {
        console.log('[Backend] Already initialized, skipping...');
        return;
    }

    console.log('[Backend] Initializing backend connections...');

    try {
        // Initialize download manager (sets up event listeners)
        await ensureDownloadManager();
        console.log('[Backend] Download manager initialized');

        // Load model cards from backend
        await loadModelCards();
        console.log('[Backend] Model cards loaded');

        // Set up performance event listeners
        await performanceService.setupEventListeners(
            (modelLoadMetrics) => {
                console.log('[Backend] Model load metrics:', modelLoadMetrics);
            },
            (inferenceMetrics) => {
                console.log('[Backend] Inference metrics:', inferenceMetrics);
            },
            (startupMetrics) => {
                console.log('[Backend] Startup metrics:', startupMetrics);
            },
        );
        console.log('[Backend] Performance listeners initialized');

        isInitialized = true;
        console.log('[Backend] All backend connections initialized successfully');
    } catch (error) {
        console.error('[Backend] Failed to initialize backend:', error);
        throw error;
    }
}

/**
 * Cleanup all backend connections.
 * Should be called when the application is unmounting.
 */
export function cleanupBackend(): void {
    if (!isInitialized) {
        return;
    }

    console.log('[Backend] Cleaning up backend connections...');

    // Stop download manager
    stopDownloadManager();

    // Cleanup performance listeners
    performanceService.cleanup();

    isInitialized = false;
    console.log('[Backend] Backend cleanup complete');
}

/**
 * Check if backend is initialized
 */
export function isBackendInitialized(): boolean {
    return isInitialized;
}

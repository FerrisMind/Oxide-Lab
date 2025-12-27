/**
 * Speech-to-Text Types
 * 
 * Type definitions for STT (Speech-to-Text) functionality.
 */

export type SttModelSource = 'bundled' | 'custom';

export type SttSettings = {
    source: SttModelSource;
    custom_dir: string | null;
};

export type SttDownloadRequest = {
    repo_id: string;
    revision: string | null;
    model_filename: string;
    tokenizer_filename: string;
    config_filename: string;
};

export type SttDownloadResponse = {
    model_dir: string;
};

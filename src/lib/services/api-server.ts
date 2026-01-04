import { invoke } from '@tauri-apps/api/core';

export interface ServerConfig {
    port: number;
    running: boolean;
}

export async function getServerConfig(): Promise<ServerConfig> {
    return await invoke('get_server_config');
}

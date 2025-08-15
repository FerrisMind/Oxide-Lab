import type { ChatMessage } from "$lib/chat/types";

export type ChatControllerCtx = {
  get modelPath(): string; set modelPath(v: string);
  get format(): "gguf" | "hub_gguf" | "hub_safetensors"; set format(v: "gguf" | "hub_gguf" | "hub_safetensors");
  get repoId(): string; set repoId(v: string);
  get revision(): string; set revision(v: string);
  get hubGgufFilename(): string; set hubGgufFilename(v: string);
  get prompt(): string; set prompt(v: string);
  get messages(): ChatMessage[]; set messages(v: ChatMessage[]);
  get messagesEl(): HTMLDivElement | null;
  get busy(): boolean; set busy(v: boolean);
  get isLoaded(): boolean; set isLoaded(v: boolean);
  get errorText(): string; set errorText(v: string);
  get isLoadingModel(): boolean; set isLoadingModel(v: boolean);
  get loadingProgress(): number; set loadingProgress(v: number);
  get loadingStage(): string; set loadingStage(v: string);
  get isCancelling(): boolean; set isCancelling(v: boolean);
  get isUnloadingModel(): boolean; set isUnloadingModel(v: boolean);
  get unloadingProgress(): number; set unloadingProgress(v: number);
  get temperature(): number; set temperature(v: number);
  get temperature_enabled(): boolean; set temperature_enabled(v: boolean);
  get top_k_enabled(): boolean; set top_k_enabled(v: boolean);
  get top_k_value(): number; set top_k_value(v: number);
  get top_p_enabled(): boolean; set top_p_enabled(v: boolean);
  get top_p_value(): number; set top_p_value(v: number);
  get min_p_enabled(): boolean; set min_p_enabled(v: boolean);
  get min_p_value(): number; set min_p_value(v: number);
  get repeat_penalty_enabled(): boolean; set repeat_penalty_enabled(v: boolean);
  get repeat_penalty_value(): number; set repeat_penalty_value(v: number);
  get ctx_limit_value(): number; set ctx_limit_value(v: number);
  get enable_thinking(): boolean; set enable_thinking(v: boolean);
  get use_custom_params(): boolean; set use_custom_params(v: boolean);
  // Устройство инференса
  get use_gpu(): boolean; set use_gpu(v: boolean);
  get cuda_available(): boolean; set cuda_available(v: boolean);
  get cuda_build(): boolean; set cuda_build(v: boolean);
  get current_device(): string; set current_device(v: string);
  // Offloading удалён
};



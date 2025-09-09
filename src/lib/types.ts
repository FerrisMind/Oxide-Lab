// Precision policy types for model loading
export type PrecisionPolicy = 
  | { Default: null }
  | { MemoryEfficient: null }
  | { MaximumPrecision: null };
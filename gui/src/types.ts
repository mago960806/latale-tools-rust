export type ViewId = "home" | "spf" | "ldt" | "stg" | "pack" | "data" | "settings";

export interface SpfFileEntry {
  name: string;
  size: number;
  resId: number;
}

export interface SpfInfo {
  path: string;
  version: number;
  encrypted: boolean;
  fileId: number;
  registryName: string | null;
  encoding: string;
  fileCount: number;
  totalSize: number;
  description: string;
  files: SpfFileEntry[];
}

export interface RegistryItem {
  fileId: number;
  name: string;
  version: number;
  encoding: string;
  includeDirs: string[];
}

export interface LdtInfo {
  path: string;
  databaseId: number;
  fieldCount: number;
  rowCount: number;
  totalSize: number;
  fields: { name: string; fieldType: string }[];
  rows: { primaryKey: number; values: string[] }[];
}

export interface StgInfo {
  path: string;
  stageCount: number;
  groupCount: number;
  mapCount: number;
  totalSize: number;
}

export interface OperationResult {
  outputPath: string;
  summary: string;
}

export interface DatabaseResult {
  outputPath: string;
  extractedFiles: number;
  importedTables: number;
  importedRows: number;
  skippedRows: number;
  failures: string[];
}

export interface ProgressEvent {
  operation: string;
  current: number;
  total: number;
  item: string;
}

export interface OpenRequest {
  action: "open" | "verify" | "unpack" | "convert";
  path: string;
}

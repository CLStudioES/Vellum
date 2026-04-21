export interface SessionInfo {
  userId: string;
  username: string;
}

export interface ProjectResponse {
  id: string;
  name: string;
  ownerId: string;
  role: string;
  createdAt: string;
  updatedAt: string;
}

export interface EnvEntry {
  key: string;
  value: string;
  lineNumber: number;
  isComment: boolean;
  isEmpty: boolean;
  isDuplicate: boolean;
  hasFormatError: boolean;
  isSensitive: boolean;
  expandsVariables: boolean;
}

export interface EnvFile {
  filename: string;
  entries: EnvEntry[];
}

export interface ScanResult {
  directory: string;
  folderName: string;
  files: EnvFile[];
}

export interface SaveResult {
  projectId: string;
  projectName: string;
  newCount: number;
  skippedCount: number;
}

export interface EntryPayload {
  envFile: string;
  key: string;
  encryptedValue: string;
  isSensitive: boolean;
}

export interface RemoteEntry {
  id: string;
  envFile: string;
  key: string;
  encryptedValue: string;
  isSensitive: boolean;
}

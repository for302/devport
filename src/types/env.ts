export interface EnvVariable {
  key: string;
  value: string;
  isSecret: boolean;
  comment: string | null;
}

export interface EnvProfile {
  name: string;
  fileName: string;
  variables: EnvVariable[];
}

export type EnvProfileType =
  | "development"
  | "staging"
  | "production"
  | { custom: string };

export interface ProfileInfo {
  name: string;
  fileName: string;
  profileType: EnvProfileType;
  isActive: boolean;
  variableCount: number;
  lastModified: number | null;
}

export interface ProfileDiff {
  key: string;
  valueA: string;
  valueB: string;
}

export interface ProfileComparison {
  profileA: string;
  profileB: string;
  onlyInA: string[];
  onlyInB: string[];
  differentValues: ProfileDiff[];
}

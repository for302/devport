/**
 * Standardized error handling for API responses
 */

/**
 * Error codes matching the Rust backend ErrorCode enum
 */
export enum ErrorCode {
  // General errors
  UNKNOWN = "UNKNOWN",
  INVALID_INPUT = "INVALID_INPUT",
  NOT_FOUND = "NOT_FOUND",
  ALREADY_EXISTS = "ALREADY_EXISTS",
  PERMISSION_DENIED = "PERMISSION_DENIED",
  TIMEOUT = "TIMEOUT",

  // Service errors
  SERVICE_NOT_FOUND = "SERVICE_NOT_FOUND",
  SERVICE_START_FAILED = "SERVICE_START_FAILED",
  SERVICE_STOP_FAILED = "SERVICE_STOP_FAILED",
  SERVICE_NOT_RUNNING = "SERVICE_NOT_RUNNING",
  SERVICE_ALREADY_RUNNING = "SERVICE_ALREADY_RUNNING",

  // Process errors
  PROCESS_NOT_FOUND = "PROCESS_NOT_FOUND",
  PROCESS_KILL_FAILED = "PROCESS_KILL_FAILED",
  PROCESS_START_FAILED = "PROCESS_START_FAILED",

  // Port errors
  PORT_IN_USE = "PORT_IN_USE",
  PORT_SCAN_FAILED = "PORT_SCAN_FAILED",
  INVALID_PORT = "INVALID_PORT",

  // File system errors
  FILE_NOT_FOUND = "FILE_NOT_FOUND",
  DIRECTORY_NOT_FOUND = "DIRECTORY_NOT_FOUND",
  FILE_READ_FAILED = "FILE_READ_FAILED",
  FILE_WRITE_FAILED = "FILE_WRITE_FAILED",
  PATH_INVALID = "PATH_INVALID",

  // Database errors
  DATABASE_CONNECTION_FAILED = "DATABASE_CONNECTION_FAILED",
  DATABASE_QUERY_FAILED = "DATABASE_QUERY_FAILED",
  DATABASE_NOT_FOUND = "DATABASE_NOT_FOUND",

  // Configuration errors
  CONFIG_INVALID = "CONFIG_INVALID",
  CONFIG_READ_FAILED = "CONFIG_READ_FAILED",
  CONFIG_WRITE_FAILED = "CONFIG_WRITE_FAILED",

  // Project errors
  PROJECT_NOT_FOUND = "PROJECT_NOT_FOUND",
  PROJECT_DETECTION_FAILED = "PROJECT_DETECTION_FAILED",
  PROJECT_ALREADY_EXISTS = "PROJECT_ALREADY_EXISTS",

  // Domain/Hosts errors
  DOMAIN_INVALID = "DOMAIN_INVALID",
  DOMAIN_CONFLICT = "DOMAIN_CONFLICT",
  HOSTS_FILE_FAILED = "HOSTS_FILE_FAILED",

  // Environment errors
  ENV_FILE_FAILED = "ENV_FILE_FAILED",
  ENV_PROFILE_NOT_FOUND = "ENV_PROFILE_NOT_FOUND",

  // Installer errors
  INSTALL_FAILED = "INSTALL_FAILED",
  DOWNLOAD_FAILED = "DOWNLOAD_FAILED",
  EXTRACTION_FAILED = "EXTRACTION_FAILED",
  COMPONENT_NOT_FOUND = "COMPONENT_NOT_FOUND",
}

/**
 * Structured error from the backend
 */
export interface CommandError {
  code: ErrorCode;
  message: string;
  details?: string;
  isRetryable: boolean;
}

/**
 * API error with additional context for the UI
 */
export interface ApiError {
  code: ErrorCode;
  message: string;
  userMessage: string;
  isRetryable: boolean;
  details?: string;
}

/**
 * User-friendly error messages for each error code
 */
const USER_MESSAGES: Partial<Record<ErrorCode, string>> = {
  [ErrorCode.SERVICE_NOT_FOUND]: "The requested service was not found",
  [ErrorCode.SERVICE_START_FAILED]: "Failed to start the service. Check if the executable exists.",
  [ErrorCode.SERVICE_STOP_FAILED]: "Failed to stop the service. It may have already stopped.",
  [ErrorCode.PORT_IN_USE]: "The port is already in use by another application",
  [ErrorCode.FILE_NOT_FOUND]: "The requested file was not found",
  [ErrorCode.PERMISSION_DENIED]: "Permission denied. Try running as administrator.",
  [ErrorCode.TIMEOUT]: "The operation timed out. Please try again.",
  [ErrorCode.DATABASE_CONNECTION_FAILED]: "Failed to connect to the database. Is it running?",
  [ErrorCode.PROJECT_NOT_FOUND]: "The project was not found",
  [ErrorCode.DOMAIN_CONFLICT]: "This domain is already in use",
  [ErrorCode.INSTALL_FAILED]: "Installation failed. Check logs for details.",
};

/**
 * Parse an error response into an ApiError
 */
export function parseError(error: unknown, context?: string): ApiError {
  // Check if it's already a CommandError from the backend
  if (isCommandError(error)) {
    return {
      code: error.code,
      message: error.message,
      userMessage: USER_MESSAGES[error.code] || error.message,
      isRetryable: error.isRetryable,
      details: error.details,
    };
  }

  // Handle string errors
  if (typeof error === "string") {
    const code = detectErrorCode(error);
    return {
      code,
      message: error,
      userMessage: context ? `${context}: ${error}` : error,
      isRetryable: false,
    };
  }

  // Handle Error objects
  if (error instanceof Error) {
    const code = detectErrorCode(error.message);
    return {
      code,
      message: error.message,
      userMessage: context ? `${context}: ${error.message}` : error.message,
      isRetryable: false,
    };
  }

  // Fallback for unknown error types
  return {
    code: ErrorCode.UNKNOWN,
    message: String(error),
    userMessage: context || "An unexpected error occurred",
    isRetryable: false,
  };
}

/**
 * Type guard to check if an error is a CommandError
 */
export function isCommandError(error: unknown): error is CommandError {
  return (
    typeof error === "object" &&
    error !== null &&
    "code" in error &&
    "message" in error &&
    typeof (error as CommandError).code === "string" &&
    typeof (error as CommandError).message === "string"
  );
}

/**
 * Detect error code from error message patterns
 */
function detectErrorCode(message: string): ErrorCode {
  const lowercased = message.toLowerCase();

  if (lowercased.includes("not found")) return ErrorCode.NOT_FOUND;
  if (lowercased.includes("permission denied")) return ErrorCode.PERMISSION_DENIED;
  if (lowercased.includes("timeout") || lowercased.includes("timed out")) return ErrorCode.TIMEOUT;
  if (lowercased.includes("port") && lowercased.includes("use")) return ErrorCode.PORT_IN_USE;
  if (lowercased.includes("service")) {
    if (lowercased.includes("start")) return ErrorCode.SERVICE_START_FAILED;
    if (lowercased.includes("stop")) return ErrorCode.SERVICE_STOP_FAILED;
  }
  if (lowercased.includes("database") || lowercased.includes("connection")) {
    return ErrorCode.DATABASE_CONNECTION_FAILED;
  }

  return ErrorCode.UNKNOWN;
}

/**
 * Check if an error is retryable
 */
export function isRetryableError(error: ApiError | ErrorCode): boolean {
  const code = typeof error === "object" ? error.code : error;
  const retryableCodes = [
    ErrorCode.TIMEOUT,
    ErrorCode.DATABASE_CONNECTION_FAILED,
    ErrorCode.SERVICE_START_FAILED,
  ];
  return retryableCodes.includes(code);
}

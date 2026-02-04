/**
 * Timing constants for the application
 * Centralized location for all delay/interval/timeout values
 */

/**
 * Auto-refresh intervals (in milliseconds)
 */
export const REFRESH_INTERVALS = {
  /** Interval for checking service health status */
  HEALTH_CHECK: 5000,
  /** Interval for refreshing service list */
  SERVICE_LIST: 10000,
  /** Interval for refreshing project list */
  PROJECT_LIST: 30000,
  /** Interval for checking phpMyAdmin availability */
  PHPMYADMIN_STATUS: 15000,
  /** Interval for log streaming updates */
  LOG_STREAM: 1000,
} as const;

/**
 * Process operation delays (in milliseconds)
 */
export const PROCESS_DELAYS = {
  /** Delay after stopping a service before starting it (restart) */
  RESTART_WAIT: 1000,
  /** Delay after killing a process before checking status */
  KILL_WAIT: 500,
  /** Initial delay before starting auto-start services */
  AUTO_START_INITIAL: 2000,
} as const;

/**
 * UI debounce delays (in milliseconds)
 */
export const DEBOUNCE_DELAYS = {
  /** Debounce for search input */
  SEARCH: 300,
  /** Debounce for port availability check */
  PORT_CHECK: 500,
  /** Debounce for domain availability check */
  DOMAIN_CHECK: 500,
  /** Debounce for form validation */
  FORM_VALIDATION: 200,
} as const;

/**
 * Animation durations (in milliseconds)
 */
export const ANIMATION_DURATIONS = {
  /** Duration for toast/notification display */
  TOAST: 3000,
  /** Duration for modal open/close animations */
  MODAL: 200,
  /** Duration for loading spinner minimum display */
  LOADING_MIN: 500,
} as const;

/**
 * Timeout values (in milliseconds)
 */
export const TIMEOUTS = {
  /** Timeout for API requests */
  API_REQUEST: 30000,
  /** Timeout for service start operation */
  SERVICE_START: 60000,
  /** Timeout for service stop operation */
  SERVICE_STOP: 30000,
  /** Timeout for health check requests */
  HEALTH_CHECK: 5000,
  /** Timeout for file operations */
  FILE_OPERATION: 10000,
} as const;

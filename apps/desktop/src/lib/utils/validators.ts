/**
 * src/lib/utils/validators.ts — Validation helpers for 900CRM.
 *
 * All validators return a { valid, error? } object.
 * Error strings are in English; callers should pass them through t() if needed.
 *
 * @module utils/validators
 */

/** Standard validation result. */
export interface ValidationResult {
  valid: boolean;
  error?: string;
}

// ─────────────────────────────────────────────────────────────────────────────
// Email
// ─────────────────────────────────────────────────────────────────────────────

/**
 * Validate an email address using RFC 5322 simplified regex.
 *
 * @param email  The email string to validate
 * @returns      ValidationResult
 */
export function validateEmail(email: string | null | undefined): ValidationResult {
  if (!email || !email.trim()) {
    return { valid: false, error: 'Email is required' };
  }

  // RFC 5322 simplified — covers 99%+ of real email addresses
  const re = /^[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*\.[a-zA-Z]{2,}$/;

  if (!re.test(email.trim())) {
    return { valid: false, error: 'Invalid email address' };
  }

  return { valid: true };
}

// ─────────────────────────────────────────────────────────────────────────────
// Phone
// ─────────────────────────────────────────────────────────────────────────────

/**
 * Validate a phone number.
 * Accepts international format with +, spaces, dashes, and parentheses.
 * Minimum 7 digits, maximum 15 (E.164 standard).
 *
 * @param phone  The phone string to validate
 * @returns      ValidationResult
 */
export function validatePhone(phone: string | null | undefined): ValidationResult {
  if (!phone || !phone.trim()) {
    return { valid: false, error: 'Phone number is required' };
  }

  const digits = phone.replace(/\D/g, '');

  if (digits.length < 7) {
    return { valid: false, error: 'Phone number is too short' };
  }

  if (digits.length > 15) {
    return { valid: false, error: 'Phone number is too long' };
  }

  // Allow +, digits, spaces, dashes, dots, and parentheses
  const re = /^[+\d][\d\s\-().]{6,}$/;
  if (!re.test(phone.trim())) {
    return { valid: false, error: 'Invalid phone number format' };
  }

  return { valid: true };
}

// ─────────────────────────────────────────────────────────────────────────────
// Required
// ─────────────────────────────────────────────────────────────────────────────

/**
 * Validate that a value is non-empty.
 *
 * @param value      The value to check (any type)
 * @param fieldName  Human-readable field name for the error message
 * @returns          ValidationResult
 */
export function validateRequired(
  value: unknown,
  fieldName = 'This field'
): ValidationResult {
  if (value === null || value === undefined) {
    return { valid: false, error: `${fieldName} is required` };
  }

  if (typeof value === 'string' && value.trim() === '') {
    return { valid: false, error: `${fieldName} is required` };
  }

  if (Array.isArray(value) && value.length === 0) {
    return { valid: false, error: `${fieldName} is required` };
  }

  return { valid: true };
}

// ─────────────────────────────────────────────────────────────────────────────
// URL
// ─────────────────────────────────────────────────────────────────────────────

/**
 * Validate a URL.
 * Must have a valid protocol (http, https, ftp, ftps).
 * Does NOT require a TLD — works with local network URLs.
 *
 * @param url  The URL string to validate
 * @returns    ValidationResult
 */
export function validateUrl(url: string | null | undefined): ValidationResult {
  if (!url || !url.trim()) {
    return { valid: false, error: 'URL is required' };
  }

  try {
    const parsed = new URL(url.trim());
    const allowed = ['http:', 'https:', 'ftp:', 'ftps:'];
    if (!allowed.includes(parsed.protocol)) {
      return { valid: false, error: 'URL must use http or https' };
    }
    return { valid: true };
  } catch {
    return { valid: false, error: 'Invalid URL' };
  }
}

// ─────────────────────────────────────────────────────────────────────────────
// String length
// ─────────────────────────────────────────────────────────────────────────────

/**
 * Validate string length within bounds.
 *
 * @param value  The string to check
 * @param min    Minimum length (inclusive)
 * @param max    Maximum length (inclusive)
 * @param field  Field name for error messages
 * @returns      ValidationResult
 */
export function validateLength(
  value: string,
  min: number,
  max: number,
  field = 'This field'
): ValidationResult {
  const len = value?.trim().length ?? 0;

  if (len < min) {
    return { valid: false, error: `${field} must be at least ${min} characters` };
  }

  if (len > max) {
    return { valid: false, error: `${field} must be at most ${max} characters` };
  }

  return { valid: true };
}

// ─────────────────────────────────────────────────────────────────────────────
// Number range
// ─────────────────────────────────────────────────────────────────────────────

/**
 * Validate a numeric value within a range.
 *
 * @param value  The number to check
 * @param min    Minimum value (inclusive)
 * @param max    Maximum value (inclusive)
 * @param field  Field name for error messages
 * @returns      ValidationResult
 */
export function validateRange(
  value: number,
  min: number,
  max: number,
  field = 'Value'
): ValidationResult {
  if (isNaN(value)) {
    return { valid: false, error: `${field} must be a number` };
  }
  if (value < min) {
    return { valid: false, error: `${field} must be at least ${min}` };
  }
  if (value > max) {
    return { valid: false, error: `${field} must be at most ${max}` };
  }
  return { valid: true };
}

// ─────────────────────────────────────────────────────────────────────────────
// Composite
// ─────────────────────────────────────────────────────────────────────────────

/**
 * Run multiple validators and return the first failure, or { valid: true }.
 *
 * @param validators  Array of ValidationResult objects
 * @returns           First failing result, or { valid: true }
 */
export function validateAll(...validators: ValidationResult[]): ValidationResult {
  for (const result of validators) {
    if (!result.valid) return result;
  }
  return { valid: true };
}

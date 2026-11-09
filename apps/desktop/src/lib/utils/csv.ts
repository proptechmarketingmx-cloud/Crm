/**
 * src/lib/utils/csv.ts — Client-side CSV parsing and building for 900CRM.
 *
 * Used to preview import files before sending data to the Rust backend.
 * Handles quoted fields, escaped quotes, and various line endings.
 *
 * @module utils/csv
 */

// ─────────────────────────────────────────────────────────────────────────────
// Types
// ─────────────────────────────────────────────────────────────────────────────

/** A parsed CSV row: header name → field value. */
export type CSVRow = Record<string, string>;

/** Result of parseCSV. */
export interface ParseCSVResult {
  /** Column headers (first row). */
  headers: string[];
  /** Data rows (excluding header). */
  rows: CSVRow[];
  /** Total row count (excluding header). */
  count: number;
  /** Any parsing warnings. */
  warnings: string[];
}

/** Column mapping: source header → target field name, or null to skip. */
export type ColumnMapping = Record<string, string | null>;

// ─────────────────────────────────────────────────────────────────────────────
// Parser
// ─────────────────────────────────────────────────────────────────────────────

/**
 * Parse a CSV string into structured rows.
 *
 * Handles:
 *   - Quoted fields (double-quote, with escaped "" inside)
 *   - Fields with commas and newlines inside quotes
 *   - Windows (\r\n), Unix (\n), and Mac (\r) line endings
 *   - Empty fields and trailing commas
 *
 * @param text  Raw CSV text
 * @returns     ParseCSVResult
 */
export function parseCSV(text: string): ParseCSVResult {
  const warnings: string[] = [];

  if (!text || !text.trim()) {
    return { headers: [], rows: [], count: 0, warnings: ['Empty input'] };
  }

  // Normalize line endings
  const normalized = text.replace(/\r\n/g, '\n').replace(/\r/g, '\n');

  const allRows = splitCSVRows(normalized);

  if (allRows.length === 0) {
    return { headers: [], rows: [], count: 0, warnings: ['No rows found'] };
  }

  const headers = parseCSVRow(allRows[0]);

  if (headers.length === 0) {
    return { headers: [], rows: [], count: 0, warnings: ['No headers found'] };
  }

  const rows: CSVRow[] = [];

  for (let i = 1; i < allRows.length; i++) {
    const raw = allRows[i].trim();
    if (!raw) continue;

    const cells = parseCSVRow(raw);

    if (cells.length !== headers.length) {
      warnings.push(`Row ${i + 1}: expected ${headers.length} columns, got ${cells.length}`);
    }

    const row: CSVRow = {};
    for (let j = 0; j < headers.length; j++) {
      row[headers[j]] = cells[j] ?? '';
    }

    rows.push(row);
  }

  return { headers, rows, count: rows.length, warnings };
}

/**
 * Split CSV text into logical rows, respecting quoted multi-line fields.
 */
function splitCSVRows(text: string): string[] {
  const rows: string[] = [];
  let current = '';
  let inQuotes = false;

  for (let i = 0; i < text.length; i++) {
    const ch = text[i];

    if (ch === '"') {
      if (inQuotes && text[i + 1] === '"') {
        // Escaped quote ""
        current += '"';
        i++;
      } else {
        inQuotes = !inQuotes;
        current += ch;
      }
    } else if (ch === '\n' && !inQuotes) {
      rows.push(current);
      current = '';
    } else {
      current += ch;
    }
  }

  if (current) rows.push(current);

  return rows;
}

/**
 * Parse a single CSV row into an array of field values.
 * Strips surrounding quotes and unescapes "" → ".
 */
function parseCSVRow(row: string): string[] {
  const fields: string[] = [];
  let current = '';
  let inQuotes = false;
  let i = 0;

  while (i < row.length) {
    const ch = row[i];

    if (ch === '"') {
      if (inQuotes && row[i + 1] === '"') {
        current += '"';
        i += 2;
        continue;
      }
      inQuotes = !inQuotes;
      i++;
      continue;
    }

    if (ch === ',' && !inQuotes) {
      fields.push(current);
      current = '';
      i++;
      continue;
    }

    current += ch;
    i++;
  }

  fields.push(current);
  return fields;
}

// ─────────────────────────────────────────────────────────────────────────────
// Builder
// ─────────────────────────────────────────────────────────────────────────────

/**
 * Build a CSV string from an array of row objects and column definitions.
 *
 * @param rows     Array of data objects
 * @param columns  Array of keys to include as columns (in order)
 * @returns        CSV string including header row
 */
export function buildCSV(rows: Record<string, unknown>[], columns: string[]): string {
  const escape = (value: unknown): string => {
    const str = value == null ? '' : String(value);
    if (str.includes(',') || str.includes('"') || str.includes('\n')) {
      return `"${str.replace(/"/g, '""')}"`;
    }
    return str;
  };

  const headerRow = columns.map(escape).join(',');
  const dataRows = rows.map((row) =>
    columns.map((col) => escape(row[col])).join(',')
  );

  return [headerRow, ...dataRows].join('\n');
}

// ─────────────────────────────────────────────────────────────────────────────
// Column mapping
// ─────────────────────────────────────────────────────────────────────────────

/**
 * Suggest an automatic column mapping by fuzzy-matching source headers
 * to target field names.
 *
 * Checks for exact matches (case-insensitive), then partial matches.
 *
 * @param sourceHeaders  Column names from the uploaded CSV
 * @param targetFields   CRM field names to map to
 * @returns              ColumnMapping suggestion (may have nulls for unmapped)
 */
export function mapColumns(
  sourceHeaders: string[],
  targetFields: string[]
): ColumnMapping {
  const mapping: ColumnMapping = {};

  for (const header of sourceHeaders) {
    const normalized = header.toLowerCase().trim();

    // Try exact match first
    const exact = targetFields.find((f) => f.toLowerCase() === normalized);
    if (exact) {
      mapping[header] = exact;
      continue;
    }

    // Try partial match (header contains field or field contains header)
    const partial = targetFields.find(
      (f) =>
        normalized.includes(f.toLowerCase()) ||
        f.toLowerCase().includes(normalized)
    );

    mapping[header] = partial ?? null;
  }

  return mapping;
}

/**
 * Apply a column mapping to CSV rows, renaming keys and dropping nulls.
 *
 * @param rows     Parsed CSV rows
 * @param mapping  Column mapping (source header → target field or null)
 * @returns        Rows with remapped keys
 */
export function applyMapping(
  rows: CSVRow[],
  mapping: ColumnMapping
): Record<string, string>[] {
  return rows.map((row) => {
    const remapped: Record<string, string> = {};
    for (const [source, target] of Object.entries(mapping)) {
      if (target !== null && source in row) {
        remapped[target] = row[source];
      }
    }
    return remapped;
  });
}

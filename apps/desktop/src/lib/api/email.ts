/**
 * src/lib/api/email.ts — Optional email integration helpers.
 */

import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-shell';

export type EmailProtocol = 'smtp' | 'imap';

export interface EmailConnectionTestRequest {
  protocol: EmailProtocol;
  host: string;
  port: number;
  timeoutMs?: number;
}

interface BackendEmailConnectionTestResult {
  protocol: EmailProtocol;
  host: string;
  port: number;
  success: boolean;
  latency_ms: number;
  details: string;
  banner?: string | null;
}

export interface EmailConnectionTestResult {
  protocol: EmailProtocol;
  host: string;
  port: number;
  success: boolean;
  latencyMs: number;
  details: string;
  banner: string | null;
}

export async function testEmailServerConnection(
  request: EmailConnectionTestRequest
): Promise<EmailConnectionTestResult> {
  const response = await invoke<BackendEmailConnectionTestResult>(
    'test_email_server_connection',
    {
      request: {
        protocol: request.protocol,
        host: request.host,
        port: request.port,
        timeout_ms: request.timeoutMs,
      },
    }
  );

  return {
    protocol: response.protocol,
    host: response.host,
    port: response.port,
    success: response.success,
    latencyMs: response.latency_ms,
    details: response.details,
    banner: response.banner ?? null,
  };
}

export interface ComposeEmailRequest {
  to: string;
  subject?: string;
  body?: string;
}

/**
 * Opens the system mail client using a `mailto:` URI.
 */
export async function composeEmail(request: ComposeEmailRequest): Promise<void> {
  const to = request.to.trim();
  if (!to) {
    throw new Error('Recipient email is required');
  }

  const params = new URLSearchParams();
  if (request.subject?.trim()) params.set('subject', request.subject.trim());
  if (request.body?.trim()) params.set('body', request.body.trim());

  const query = params.toString();
  const url = `mailto:${encodeURIComponent(to)}${query ? `?${query}` : ''}`;
  await open(url);
}

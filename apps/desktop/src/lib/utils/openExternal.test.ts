import { beforeEach, describe, expect, it, vi } from 'vitest';

const { openMock, invokeMock } = vi.hoisted(() => ({
  openMock: vi.fn(),
  invokeMock: vi.fn(),
}));

vi.mock('@tauri-apps/plugin-shell', () => ({
  open: openMock,
}));

vi.mock('@tauri-apps/api/core', () => ({
  invoke: invokeMock,
}));

import { openExternalUrl, openLocalPath } from './openExternal';

describe('openExternalUrl', () => {
  beforeEach(() => {
    openMock.mockReset();
    openMock.mockResolvedValue(undefined);
  });

  it('opens http and https URLs', async () => {
    await openExternalUrl('https://northstar.example');
    expect(openMock).toHaveBeenCalledWith('https://northstar.example/');
  });

  it('rejects javascript and missing protocol URLs', async () => {
    await expect(openExternalUrl('javascript:alert(1)')).rejects.toThrow(/http or https/i);
    await expect(openExternalUrl('not-a-url')).rejects.toThrow();
    expect(openMock).not.toHaveBeenCalled();
  });
});

describe('openLocalPath', () => {
  beforeEach(() => {
    openMock.mockReset();
    invokeMock.mockReset();
    openMock.mockResolvedValue(undefined);
    invokeMock.mockImplementation(async (cmd: string, args?: { file_path?: string }) => {
      if (cmd === 'validate_open_path') {
        return args?.file_path;
      }
      throw new Error(`unexpected invoke ${cmd}`);
    });
  });

  it('opens a local filesystem path', async () => {
    await openLocalPath('/tmp/900crm-quote.pdf');
    expect(invokeMock).toHaveBeenCalledWith('validate_open_path', {
      file_path: '/tmp/900crm-quote.pdf',
    });
    expect(openMock).toHaveBeenCalledWith('/tmp/900crm-quote.pdf');
  });

  it('rejects URL schemes', async () => {
    await expect(openLocalPath('https://evil.example/file')).rejects.toThrow(/local path/i);
    await expect(openLocalPath('file:///etc/passwd')).rejects.toThrow(/local path/i);
    expect(openMock).not.toHaveBeenCalled();
  });

  it('rejects paths with control characters', async () => {
    await expect(openLocalPath('/tmp/quote.pdf\n/etc/passwd')).rejects.toThrow(/invalid/i);
    expect(openMock).not.toHaveBeenCalled();
  });
});

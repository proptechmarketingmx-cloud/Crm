import { invoke } from '@tauri-apps/api/core';

export type SaveTextFileResult = 'saved' | 'downloaded' | 'cancelled';

export function downloadTextFile(filename: string, contents: string, mime = 'text/csv;charset=utf-8'): void {
  const blob = new Blob([contents], { type: mime });
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = filename;
  link.rel = 'noopener';
  document.body.append(link);
  link.click();
  link.remove();
  URL.revokeObjectURL(url);
}

export async function saveTextFile(
  filename: string,
  contents: string,
  mime = 'text/csv;charset=utf-8',
): Promise<SaveTextFileResult> {
  try {
    const { save } = await import('@tauri-apps/plugin-dialog');
    const path = await save({
      defaultPath: filename,
      filters: [{ name: 'CSV', extensions: ['csv'] }],
    });
    if (typeof path !== 'string' || !path.trim()) {
      return 'cancelled';
    }

    await invoke('write_export_text', {
      file_path: path,
      contents,
    });
    return 'saved';
  } catch {
    downloadTextFile(filename, contents, mime);
    return 'downloaded';
  }
}

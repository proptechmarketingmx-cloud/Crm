import { spawn } from 'node:child_process';

export function runCommand(command, args, options = {}) {
  const timeoutMs = Number(options.timeoutMs ?? 30_000);
  const label = options.label ?? [command, ...args].join(' ');

  return new Promise((resolve, reject) => {
    const child = spawn(command, args, {
      cwd: options.cwd,
      env: options.env,
      stdio: 'inherit'
    });

    let timedOut = false;
    const timer = setTimeout(() => {
      timedOut = true;
      child.kill('SIGTERM');
      setTimeout(() => child.kill('SIGKILL'), 2_000).unref();
    }, timeoutMs);

    child.on('error', (error) => {
      clearTimeout(timer);
      reject(error);
    });

    child.on('exit', (code, signal) => {
      clearTimeout(timer);
      if (timedOut) {
        reject(
          new Error(
            `${label} timed out after ${timeoutMs}ms. This usually means the local Svelte/Vite ESM loader is stuck before diagnostics. Run npm install from the repo root, then try node -e "import('svelte').then(() => console.log('ok'))" to isolate package loading.`
          )
        );
        return;
      }

      if (code === 0) {
        resolve();
        return;
      }

      reject(new Error(`${label} exited with code ${code ?? `signal ${signal}`}`));
    });
  });
}

// Wrapper to handle Tauri API import
export async function invoke(command, payload) {
  try {
    const tauriModule = await import('@tauri-apps/api');
    const { invoke: tauriInvoke } = tauriModule;
    return await tauriInvoke(command, payload);
  } catch (err) {
    console.error('Failed to import Tauri API:', err);
    throw err;
  }
}

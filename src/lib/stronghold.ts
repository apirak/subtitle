import { invoke } from '@tauri-apps/api/core';
import { appDataDir } from '@tauri-apps/api/path';
import { type Client, Location, Stronghold } from '@tauri-apps/plugin-stronghold';

let strongholdInstance: Stronghold | null = null;
let clientInstance: Client | null = null;

export async function getStronghold(): Promise<Stronghold> {
  if (strongholdInstance) return strongholdInstance;

  const vaultPath = await invoke<string>('stronghold_get_vault_path');
  const password = await invoke<string>('stronghold_get_password');

  strongholdInstance = await Stronghold.load(vaultPath, password);
  return strongholdInstance;
}

export async function getClient(): Promise<Client> {
  if (clientInstance) return clientInstance;

  const stronghold = await getStronghold();
  const clientName = 'api-keys';

  try {
    clientInstance = await stronghold.loadClient(clientName);
  } catch {
    clientInstance = await stronghold.createClient(clientName);
  }

  return clientInstance;
}

export async function saveApiKey(keyName: string, keyValue: string): Promise<void> {
  if (!keyValue || !keyValue.trim()) return;

  const stronghold = await getStronghold();
  const client = await getClient();
  const store = client.getStore();
  const data = Array.from(new TextEncoder().encode(keyValue));
  await store.insert(keyName, data);
  await stronghold.save();
}

export async function getApiKey(keyName: string): Promise<string | null> {
  const client = await getClient();
  const store = client.getStore();
  const data = await store.get(keyName);
  if (!data) return null;
  return new TextDecoder().decode(new Uint8Array(data));
}

export async function removeApiKey(keyName: string): Promise<void> {
  const client = await getClient();
  const store = client.getStore();
  await store.remove(keyName);
}

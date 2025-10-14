/**
 * Test Helper: Server Availability Check
 * 
 * This helper checks if the Vectorizer server is running before
 * executing integration tests.
 */

import { VectorizerClient } from '../src/client';

export async function checkServerAvailability(baseURL: string): Promise<boolean> {
  const client = new VectorizerClient({
    baseURL,
    timeout: 3000,
  });

  try {
    await client.healthCheck();
    return true;
  } catch (error) {
    return false;
  }
}

export function skipIfServerNotAvailable(serverAvailable: boolean, testFn: () => void | Promise<void>) {
  if (!serverAvailable) {
    // Skip test by passing it
    return;
  }
  return testFn();
}


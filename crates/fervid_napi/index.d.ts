/* tslint:disable */
/* eslint-disable */

/* auto-generated by NAPI-RS */

export interface CompileSyncOptions {
  isProd: boolean
}
export function compileSync(source: string, options?: CompileSyncOptions | undefined | null): string
export function compileAsync(
  source: string,
  options?: CompileSyncOptions | undefined | null,
  signal?: AbortSignal | undefined | null,
): Promise<unknown>

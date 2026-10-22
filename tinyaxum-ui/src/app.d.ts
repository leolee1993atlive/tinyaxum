// See https://svelte.dev/docs/kit/types#app.d.ts
// for information about these interfaces

// Environment Variables
interface ImportMetaEnv {
  readonly VITE_API_BASE_URL: string
  readonly VITE_USE_MOCK_API?: string
}

interface ImportMeta {
  readonly env: ImportMetaEnv
}

declare global {
  namespace App {
    interface Locals {
      user: { id: number; username: string } | null
    }
  }
}

declare module '@tanstack/table-core' {
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  interface ColumnMeta<TData extends RowData, TValue> {
    cellClass?: string
    align?: 'left' | 'center' | 'right'
  }
}

export {}

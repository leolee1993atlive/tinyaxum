import { getContext, setContext } from 'svelte'

export class CrudPageState<TRow, TDialog extends string = string> {
  open = $state<TDialog | null>(null)
  currentRow = $state<TRow | null>(null)

  setOpen(val: TDialog | null) {
    this.open = this.open === val ? null : val
  }

  setCurrentRow(row: TRow | null) {
    this.currentRow = row
  }

  confirmDelete(row: TRow) {
    this.currentRow = row
    this.open = 'delete' as TDialog
  }

  closeDialogs(delay = 200) {
    this.open = null
    setTimeout(() => {
      this.currentRow = null
    }, delay)
  }
}

export function createCrudContext<T extends CrudPageState<unknown, string>>(
  label: string
) {
  const contextKey = Symbol(`${label}-context`)

  function provide(value: T) {
    setContext(contextKey, value)
  }

  function use(): T {
    const context = getContext<T>(contextKey)
    if (!context) {
      throw new Error(`use${label} has to be used within <${label}Provider>`)
    }
    return context
  }

  return { provide, use }
}

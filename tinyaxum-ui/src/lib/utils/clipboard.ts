export async function copyText(text: string): Promise<void> {
  if (
    typeof navigator !== 'undefined' &&
    navigator.clipboard &&
    typeof window !== 'undefined' &&
    window.isSecureContext
  ) {
    await navigator.clipboard.writeText(text)
    return
  }

  // Fallback for non-secure contexts (intranet HTTP).
  // navigator.clipboard requires HTTPS or localhost; execCommand does not.
  const textarea = document.createElement('textarea')
  textarea.value = text
  textarea.setAttribute('readonly', '')
  textarea.style.position = 'fixed'
  textarea.style.top = '0'
  textarea.style.left = '0'
  textarea.style.width = '1px'
  textarea.style.height = '1px'
  textarea.style.opacity = '0'
  document.body.appendChild(textarea)
  textarea.select()
  textarea.setSelectionRange(0, text.length)
  try {
    const ok = document.execCommand('copy')
    if (!ok) throw new Error('execCommand("copy") returned false')
  } finally {
    document.body.removeChild(textarea)
  }
}

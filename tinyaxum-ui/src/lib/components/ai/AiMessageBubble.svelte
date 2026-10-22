<script lang="ts">
  import { Button } from '$lib/components/ui/button/index.js'
  import { copyText } from '$lib/utils/clipboard.js'
  import { Copy, Check, FilePenLine } from '@lucide/svelte'
  import DOMPurify from 'dompurify'
  import { marked } from 'marked'
  import { toast } from 'svelte-sonner'

  interface Props {
    role: 'user' | 'assistant'
    content: string
    /** 提供后，AI 回复中的 SQL 代码块会出现「应用到片段」按钮 */
    onApplyCode?: (code: string) => void
  }

  let { role, content, onApplyCode }: Props = $props()
  let copiedKey = $state<number | null>(null)

  marked.setOptions({
    gfm: true,
    breaks: true,
  })

  type Segment =
    | { kind: 'text'; text: string; html: string }
    | { kind: 'code'; lang: string; code: string }

  const segments = $derived(parse(content))

  function renderMarkdown(text: string): string {
    const raw = marked.parse(text, { async: false }) as string
    return DOMPurify.sanitize(raw)
  }

  function parse(s: string): Segment[] {
    const out: Segment[] = []
    const re = /```(\w*)\n([\s\S]*?)```/g
    let last = 0
    let m: RegExpExecArray | null
    while ((m = re.exec(s)) !== null) {
      if (m.index > last) {
        const text = s.slice(last, m.index)
        if (text.trim())
          out.push({ kind: 'text', text, html: renderMarkdown(text) })
      }
      out.push({ kind: 'code', lang: m[1] || 'text', code: m[2] })
      last = re.lastIndex
    }
    if (last < s.length) {
      const text = s.slice(last)
      if (text.trim())
        out.push({ kind: 'text', text, html: renderMarkdown(text) })
    }
    return out
  }

  async function copyCode(code: string, idx: number) {
    try {
      await copyText(code)
      copiedKey = idx
      toast.success('已复制')
      setTimeout(() => {
        if (copiedKey === idx) copiedKey = null
      }, 1500)
    } catch (e) {
      toast.error(`复制失败: ${e}`)
    }
  }
</script>

<div class={role === 'user' ? 'flex justify-end' : 'flex justify-start'}>
  <div
    class={role === 'user'
      ? 'bg-primary text-primary-foreground max-w-[85%] rounded-lg px-3 py-2 text-sm whitespace-pre-wrap'
      : 'bg-muted max-w-[85%] rounded-lg px-3 py-2 text-sm'}
  >
    {#each segments as seg, i (i)}
      {#if seg.kind === 'text'}
        {#if role === 'assistant'}
          <div class="md-body">{@html seg.html}</div>
        {:else}
          <span class="whitespace-pre-wrap">{seg.text}</span>
        {/if}
      {:else}
        <div class="my-2 overflow-hidden rounded border">
          <div
            class="bg-muted-foreground/10 flex items-center justify-between px-2 py-1 text-xs"
          >
            <span class="font-mono">{seg.lang}</span>
            <div class="flex items-center gap-1">
              {#if role === 'assistant' && onApplyCode && seg.lang.toLowerCase() === 'sql'}
                <Button
                  variant="ghost"
                  size="sm"
                  class="h-6 gap-1"
                  onclick={() => onApplyCode(seg.code)}
                >
                  <FilePenLine class="h-3 w-3" /> 应用到片段
                </Button>
              {/if}
              <Button
                variant="ghost"
                size="sm"
                class="h-6 gap-1"
                onclick={() => copyCode(seg.code, i)}
              >
                {#if copiedKey === i}
                  <Check class="h-3 w-3" /> 已复制
                {:else}
                  <Copy class="h-3 w-3" /> 复制
                {/if}
              </Button>
            </div>
          </div>
          <pre class="bg-background overflow-x-auto px-3 py-2 text-xs"><code
              >{seg.code}</code
            ></pre>
        </div>
      {/if}
    {/each}
  </div>
</div>

<style>
  .md-body :global(p) {
    margin: 0.25rem 0;
    line-height: 1.6;
  }
  .md-body :global(p:first-child) {
    margin-top: 0;
  }
  .md-body :global(p:last-child) {
    margin-bottom: 0;
  }
  .md-body :global(h1),
  .md-body :global(h2),
  .md-body :global(h3),
  .md-body :global(h4),
  .md-body :global(h5),
  .md-body :global(h6) {
    margin: 0.75rem 0 0.4rem;
    font-weight: 600;
    line-height: 1.3;
  }
  .md-body :global(h1) {
    font-size: 1.15rem;
  }
  .md-body :global(h2) {
    font-size: 1.05rem;
  }
  .md-body :global(h3) {
    font-size: 0.95rem;
  }
  .md-body :global(h4),
  .md-body :global(h5),
  .md-body :global(h6) {
    font-size: 0.875rem;
  }
  .md-body :global(ul),
  .md-body :global(ol) {
    margin: 0.4rem 0;
    padding-left: 1.5rem;
  }
  .md-body :global(ul) {
    list-style: disc;
  }
  .md-body :global(ol) {
    list-style: decimal;
  }
  .md-body :global(li) {
    margin: 0.15rem 0;
  }
  .md-body :global(li > ul),
  .md-body :global(li > ol) {
    margin: 0.15rem 0;
  }
  .md-body :global(a) {
    color: hsl(var(--primary, 221 83% 53%));
    text-decoration: underline;
    text-underline-offset: 2px;
  }
  .md-body :global(strong) {
    font-weight: 600;
  }
  .md-body :global(em) {
    font-style: italic;
  }
  .md-body :global(code) {
    background-color: rgb(0 0 0 / 0.08);
    padding: 0.1rem 0.3rem;
    border-radius: 0.25rem;
    font-size: 0.85em;
    font-family:
      ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  }
  :global(.dark) .md-body :global(code) {
    background-color: rgb(255 255 255 / 0.1);
  }
  .md-body :global(pre) {
    margin: 0.5rem 0;
    padding: 0.5rem 0.75rem;
    background-color: rgb(0 0 0 / 0.05);
    border-radius: 0.375rem;
    overflow-x: auto;
  }
  .md-body :global(pre code) {
    background: none;
    padding: 0;
    border-radius: 0;
    font-size: 0.85em;
  }
  .md-body :global(blockquote) {
    margin: 0.5rem 0;
    padding-left: 0.75rem;
    border-left: 3px solid hsl(var(--border, 240 5% 84%));
    color: hsl(var(--muted-foreground, 240 4% 46%));
  }
  .md-body :global(table) {
    margin: 0.5rem 0;
    border-collapse: collapse;
    font-size: 0.85em;
  }
  .md-body :global(th),
  .md-body :global(td) {
    border: 1px solid hsl(var(--border, 240 5% 84%));
    padding: 0.25rem 0.5rem;
    text-align: left;
  }
  .md-body :global(th) {
    background-color: rgb(0 0 0 / 0.04);
    font-weight: 600;
  }
  .md-body :global(hr) {
    margin: 0.75rem 0;
    border: none;
    border-top: 1px solid hsl(var(--border, 240 5% 84%));
  }
</style>

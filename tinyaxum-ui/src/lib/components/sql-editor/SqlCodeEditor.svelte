<script lang="ts">
  import { onMount, onDestroy } from 'svelte'
  import { useTheme } from '$lib/context/theme.svelte.js'
  import { cn } from '$lib/utils.js'
  import {
    defaultKeymap,
    history,
    historyKeymap,
    indentLess,
    indentMore,
  } from '@codemirror/commands'
  import { sql, StandardSQL } from '@codemirror/lang-sql'
  import {
    bracketMatching,
    HighlightStyle,
    syntaxHighlighting,
  } from '@codemirror/language'
  import { autocompletion } from '@codemirror/autocomplete'
  import { search, searchKeymap } from '@codemirror/search'
  import {
    Compartment,
    EditorState,
    type StateCommand,
  } from '@codemirror/state'
  import {
    EditorView,
    keymap,
    lineNumbers,
    highlightActiveLineGutter,
    highlightActiveLine,
    type ViewUpdate,
  } from '@codemirror/view'
  import { tags as t } from '@lezer/highlight'
  import type { DatasourceTableSchemaVo } from '$lib/api/types.js'
  import { sqlQueriesApi } from '$lib/api/sqlqueries.js'
  import { buildCmSchema } from './sql-schema-completion.js'

  type SqlValidateResult = { errors: string[]; warnings: string[] }

  type Props = {
    value?: string
    onchange?: (value: string) => void
    onrun?: () => void
    class?: string
    schema?: DatasourceTableSchemaVo[]
    dataSourceId?: number
    onvalidate?: (result: SqlValidateResult) => void
  }

  let {
    value = $bindable(''),
    onchange,
    onrun,
    class: className,
    schema = [],
    dataSourceId = 0,
    onvalidate,
  }: Props = $props()

  const theme = useTheme()
  let editorContainer: HTMLDivElement
  let view: EditorView | undefined = $state()
  let themeCompartment = new Compartment()
  let highlightCompartment = new Compartment()
  let schemaCompartment = new Compartment()

  // Syntax highlight for dark theme (VS Code dark+ inspired)
  const darkHighlightStyle = HighlightStyle.define([
    { tag: t.keyword, color: '#569cd6', fontWeight: 'bold' },
    { tag: [t.controlKeyword, t.moduleKeyword], color: '#c586c0' },
    { tag: [t.operator, t.operatorKeyword], color: '#d4d4d4' },
    { tag: [t.string, t.special(t.string)], color: '#ce9178' },
    { tag: t.number, color: '#b5cea8' },
    { tag: t.bool, color: '#569cd6' },
    { tag: t.null, color: '#569cd6' },
    {
      tag: [t.function(t.variableName), t.function(t.propertyName)],
      color: '#dcdcaa',
    },
    { tag: [t.variableName, t.propertyName], color: '#9cdcfe' },
    { tag: [t.typeName, t.className], color: '#4ec9b0' },
    { tag: t.comment, color: '#6a9955', fontStyle: 'italic' },
    { tag: t.punctuation, color: '#d4d4d4' },
    { tag: t.bracket, color: '#ffd700' },
    { tag: t.invalid, color: '#f48771' },
  ])

  // Syntax highlight for light theme
  const lightHighlightStyle = HighlightStyle.define([
    { tag: t.keyword, color: '#0000ff', fontWeight: 'bold' },
    { tag: [t.controlKeyword, t.moduleKeyword], color: '#af00db' },
    { tag: [t.operator, t.operatorKeyword], color: '#000000' },
    { tag: [t.string, t.special(t.string)], color: '#a31515' },
    { tag: t.number, color: '#098658' },
    { tag: t.bool, color: '#0000ff' },
    { tag: t.null, color: '#0000ff' },
    {
      tag: [t.function(t.variableName), t.function(t.propertyName)],
      color: '#795e26',
    },
    { tag: [t.variableName, t.propertyName], color: '#001080' },
    { tag: [t.typeName, t.className], color: '#267f99' },
    { tag: t.comment, color: '#008000', fontStyle: 'italic' },
    { tag: t.punctuation, color: '#000000' },
    { tag: t.bracket, color: '#319331' },
    { tag: t.invalid, color: '#cd3131' },
  ])

  function getHighlightExtension() {
    return syntaxHighlighting(
      theme.resolvedTheme === 'dark' ? darkHighlightStyle : lightHighlightStyle
    )
  }

  function getSchemaExtensions() {
    const cmSchema = buildCmSchema(schema)
    return [
      sql({ dialect: StandardSQL, schema: cmSchema }),
      autocompletion({
        activateOnTyping: true,
        maxRenderedOptions: 50,
      }),
    ]
  }

  // Dark theme matching the app's dark palette
  const darkTheme = EditorView.theme(
    {
      '&': {
        backgroundColor: '#09090b',
        color: '#fafafa',
        height: '100%',
      },
      // 编辑器自身成为滚动容器：长 SQL 走视口虚拟化，光标 reveal 不再波及祖先
      '.cm-scroller': {
        overflow: 'auto',
      },
      '.cm-content': {
        caretColor: '#fafafa',
        fontFamily: "'JetBrains Mono', 'Fira Code', 'Cascadia Code', monospace",
        fontSize: '13px',
        lineHeight: '1.6',
      },
      '.cm-cursor': {
        borderLeftColor: '#fafafa',
      },
      '&.cm-focused .cm-selectionBackground, .cm-selectionBackground, .cm-content ::selection':
        {
          backgroundColor: '#264f78',
        },
      '.cm-gutters': {
        backgroundColor: '#09090b',
        color: '#71717a',
        borderRight: '1px solid #27272a',
      },
      '.cm-activeLineGutter': {
        backgroundColor: '#18181b',
        color: '#a1a1aa',
      },
      '.cm-activeLine': {
        backgroundColor: '#18181b',
      },
      '.cm-matchingBracket': {
        backgroundColor: '#3f3f46',
        outline: '1px solid #71717a',
        color: '#fafafa !important',
      },
      '.cm-selectionMatch': {
        backgroundColor: '#3f3f46',
      },
      '.cm-line': {
        padding: '0 4px',
      },
      '.cm-panels': {
        backgroundColor: '#18181b',
        color: '#fafafa',
        borderBottom: '1px solid #27272a',
        zIndex: '10',
      },
      '.cm-panel.cm-search': {
        padding: '6px 8px',
      },
      '.cm-panel.cm-search input, .cm-panel.cm-search button': {
        backgroundColor: '#09090b',
        color: '#fafafa',
        border: '1px solid #27272a',
        borderRadius: '4px',
        padding: '2px 6px',
      },
      '.cm-panel.cm-search button:hover': {
        backgroundColor: '#27272a',
      },
      '.cm-panel.cm-search label': {
        color: '#a1a1aa',
      },
      '.cm-searchMatch': {
        backgroundColor: '#facc1540',
        outline: '1px solid #facc15',
      },
      '.cm-searchMatch-selected': {
        backgroundColor: '#facc1580',
      },
    },
    { dark: true }
  )

  // Light theme
  const lightTheme = EditorView.theme({
    '&': {
      backgroundColor: '#ffffff',
      color: '#09090b',
      height: '100%',
    },
    '.cm-scroller': {
      overflow: 'auto',
    },
    '.cm-content': {
      caretColor: '#09090b',
      fontFamily: "'JetBrains Mono', 'Fira Code', 'Cascadia Code', monospace",
      fontSize: '13px',
      lineHeight: '1.6',
    },
    '.cm-cursor': {
      borderLeftColor: '#09090b',
    },
    '&.cm-focused .cm-selectionBackground, .cm-selectionBackground, .cm-content ::selection':
      {
        backgroundColor: '#bfdbfe',
      },
    '.cm-gutters': {
      backgroundColor: '#fafafa',
      color: '#a1a1aa',
      borderRight: '1px solid #e4e4e7',
    },
    '.cm-activeLineGutter': {
      backgroundColor: '#f4f4f5',
      color: '#71717a',
    },
    '.cm-activeLine': {
      backgroundColor: '#f4f4f5',
    },
    '.cm-matchingBracket': {
      backgroundColor: '#e4e4e7',
      outline: '1px solid #a1a1aa',
      color: '#09090b !important',
    },
    '.cm-selectionMatch': {
      backgroundColor: '#e4e4e7',
    },
    '.cm-line': {
      padding: '0 4px',
    },
    '.cm-panels': {
      backgroundColor: '#fafafa',
      color: '#09090b',
      borderBottom: '1px solid #e4e4e7',
      zIndex: '10',
    },
    '.cm-panel.cm-search': {
      padding: '6px 8px',
    },
    '.cm-panel.cm-search input, .cm-panel.cm-search button': {
      backgroundColor: '#ffffff',
      color: '#09090b',
      border: '1px solid #e4e4e7',
      borderRadius: '4px',
      padding: '2px 6px',
    },
    '.cm-panel.cm-search button:hover': {
      backgroundColor: '#f4f4f5',
    },
    '.cm-panel.cm-search label': {
      color: '#71717a',
    },
    '.cm-searchMatch': {
      backgroundColor: '#fef08a',
      outline: '1px solid #eab308',
    },
    '.cm-searchMatch-selected': {
      backgroundColor: '#fde047',
    },
  })

  function getThemeExtension() {
    return theme.resolvedTheme === 'dark' ? darkTheme : lightTheme
  }

  const searchPhrases: Record<string, string> = {
    Find: '查找',
    Replace: '替换',
    next: '下一个',
    previous: '上一个',
    all: '全部',
    'match case': '区分大小写',
    'by word': '全词匹配',
    regexp: '正则',
    replace: '替换',
    'replace all': '全部替换',
    close: '关闭',
    'current match': '当前匹配',
    'on line': '所在行',
    'Go to line': '跳转到行',
    go: '跳转',
    'replaced $ matches': '已替换 $ 处',
    'replaced match on line $': '已替换第 $ 行匹配',
  }

  /** Tab = 2 spaces; avoid browser focus trap. Multi-line selection uses indent/outdent. */
  const insertTwoSpacesTab: StateCommand = ({ state, dispatch }) => {
    if (state.selection.ranges.some((r) => !r.empty)) {
      return indentMore({ state, dispatch })
    }
    dispatch(
      state.update(state.replaceSelection('  '), {
        scrollIntoView: true,
        userEvent: 'input',
      })
    )
    return true
  }

  onMount(() => {
    const state = EditorState.create({
      doc: value,
      extensions: [
        EditorState.phrases.of(searchPhrases),
        lineNumbers(),
        highlightActiveLineGutter(),
        highlightActiveLine(),
        bracketMatching(),
        history(),
        search({ top: true }),
        schemaCompartment.of(getSchemaExtensions()),
        keymap.of([
          {
            key: 'Tab',
            run: insertTwoSpacesTab,
            shift: indentLess,
            preventDefault: true,
          },
          {
            key: 'Ctrl-Enter',
            run: () => {
              onrun?.()
              return true
            },
          },
          ...searchKeymap,
          ...defaultKeymap,
          ...historyKeymap,
        ]),
        EditorView.updateListener.of((update: ViewUpdate) => {
          if (update.docChanged) {
            const newValue = update.state.doc.toString()
            value = newValue
            onchange?.(newValue)
          }
        }),
        EditorView.domEventHandlers({
          blur: () => {
            if (validateTimer) clearTimeout(validateTimer)
            void doValidate(value)
          },
        }),
        themeCompartment.of(getThemeExtension()),
        highlightCompartment.of(getHighlightExtension()),
        EditorView.lineWrapping,
      ],
    })

    view = new EditorView({
      state,
      parent: editorContainer,
    })
  })

  // Sync external value changes into the editor
  $effect(() => {
    if (!view) return
    const currentDoc = view.state.doc.toString()
    if (value !== currentDoc) {
      view.dispatch({
        changes: { from: 0, to: currentDoc.length, insert: value },
      })
    }
  })

  // React to theme changes
  $effect(() => {
    if (!view) return
    const _resolved = theme.resolvedTheme
    view.dispatch({
      effects: [
        themeCompartment.reconfigure(getThemeExtension()),
        highlightCompartment.reconfigure(getHighlightExtension()),
      ],
    })
  })

  $effect(() => {
    if (!view) return
    const _ = schema
    view.dispatch({
      effects: schemaCompartment.reconfigure(getSchemaExtensions()),
    })
  })

  let validateTimer: ReturnType<typeof setTimeout> | null = null
  let isValidating = $state(false)

  async function doValidate(sql: string) {
    if (!onvalidate || !sql.trim() || dataSourceId <= 0) {
      onvalidate?.({ errors: [], warnings: [] })
      return
    }
    if (isValidating) return
    isValidating = true
    try {
      const res = await sqlQueriesApi.validate(sql, dataSourceId)
      if (res.data) {
        onvalidate?.({ errors: [res.data], warnings: [] })
      } else {
        onvalidate?.({ errors: [], warnings: [] })
      }
    } catch {
      onvalidate?.({ errors: [], warnings: [] })
    } finally {
      isValidating = false
    }
  }

  $effect(() => {
    const sql = value
    const _ = schema
    const _ds = dataSourceId
    if (!onvalidate || !sql.trim() || dataSourceId <= 0) {
      onvalidate?.({ errors: [], warnings: [] })
      return
    }
    if (validateTimer) clearTimeout(validateTimer)
    validateTimer = setTimeout(() => {
      void doValidate(sql)
    }, 800)
  })

  onDestroy(() => {
    view?.destroy()
  })

  export function insertAtCursor(text: string) {
    if (!view) return
    const cursor = view.state.selection.main.head
    view.dispatch({
      changes: { from: cursor, insert: text },
      selection: { anchor: cursor + text.length },
    })
    view.focus()
  }
</script>

<div
  data-slot="sql-code-editor"
  class={cn(
    'bg-background relative flex flex-col overflow-hidden rounded-md border',
    className
  )}
>
  <div bind:this={editorContainer} class="min-h-0 flex-1"></div>
</div>

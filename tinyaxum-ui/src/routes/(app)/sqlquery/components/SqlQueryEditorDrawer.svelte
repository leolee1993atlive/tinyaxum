<script lang="ts">
  import { onDestroy, onMount, tick } from 'svelte'
  import { isApiClientNotifiedError } from '$lib/api/client.js'
  import { datasourcesApi } from '$lib/api/datasources.js'
  import { sqlQueriesApi } from '$lib/api/sqlqueries.js'
  import { sqlSnippetsApi } from '$lib/api/sqlsnippets.js'
  import type {
    DatasourceListVo,
    DatasourceTableSchemaVo,
    ModuleOptionVo,
    QueryResultRow,
    SqlQueryCollectionFlag,
    SqlQueryDetailVo,
    SqlQueryListVo,
    SqlQueryResultFieldVo,
    SqlQueryVizConfig,
  } from '$lib/api/types.js'
  import SelectDropdown from '$lib/components/SelectDropdown.svelte'
  import AiChatDrawer from '$lib/components/ai/AiChatDrawer.svelte'
  import SqlVisualizationDrawer from '$lib/components/sql-editor/SqlVisualizationDrawer.svelte'
  import * as AlertDialog from '$lib/components/ui/alert-dialog/index.js'
  import { Badge } from '$lib/components/ui/badge/index.js'
  import { Button } from '$lib/components/ui/button/index.js'
  import * as Card from '$lib/components/ui/card/index.js'
  import { Checkbox } from '$lib/components/ui/checkbox/index.js'
  import * as Dialog from '$lib/components/ui/dialog/index.js'
  import { Input } from '$lib/components/ui/input/index.js'
  import { Label } from '$lib/components/ui/label/index.js'
  import * as Sheet from '$lib/components/ui/sheet/index.js'
  import { normalizeVizConfig } from '$lib/sql-query/viz-config.js'
  import { buildSnapshot } from '$lib/utils/dirty-snapshot.js'
  import { formatRelativeTime } from '$lib/utils/relative-time.js'
  import {
    SPLIT_DEFAULT,
    SPLIT_MAX,
    SPLIT_MIN,
    clampSplit,
    loadSplit,
    saveSplit,
  } from '$lib/utils/split-persist.js'
  import { formatSql } from '$lib/utils/sql-format.js'
  import {
    expandSnippets,
    extractParams,
    extractSnippetRefs,
  } from '$lib/utils/sql-params.js'
  import {
    Play,
    Save,
    LoaderCircle,
    Wand2,
    Settings2,
    Bot,
    Eye,
    X,
  } from '@lucide/svelte'
  import { toast } from 'svelte-sonner'
  import { useSQLQuery } from './sqlquery-state.svelte'

  const queryState = useSQLQuery()

  let {
    dataSourceId = $bindable(0),
    moduleOptions = [],
  }: { dataSourceId?: number; moduleOptions?: ModuleOptionVo[] } = $props()

  // Dynamically import components to avoid SSR issues with CodeMirror
  type SqlCodeEditorType =
    typeof import('$lib/components/sql-editor/SqlCodeEditor.svelte').default
  type QueryResultsTableType =
    typeof import('$lib/components/sql-editor/QueryResultsTable.svelte').default
  type SnippetSidebarType =
    typeof import('$lib/components/sql-editor/SnippetSidebar.svelte').default
  type SnippetSidebarInstance = ReturnType<SnippetSidebarType> | null
  type SchemaSidebarType =
    typeof import('$lib/components/sql-editor/SchemaSidebar.svelte').default
  type QuerySelectorType =
    typeof import('$lib/components/sql-editor/QuerySelector.svelte').default
  type TableDataDrawerType =
    typeof import('$lib/components/sql-editor/TableDataDrawer.svelte').default

  let SqlCodeEditor: SqlCodeEditorType | null = $state(null)
  let QueryResultsTable: QueryResultsTableType | null = $state(null)
  let SnippetSidebar: SnippetSidebarType | null = $state(null)
  let snippetSidebarRef: SnippetSidebarInstance = $state(null)
  let SchemaSidebar: SchemaSidebarType | null = $state(null)
  let QuerySelector: QuerySelectorType | null = $state(null)
  let TableDataDrawer: TableDataDrawerType | null = $state(null)

  // State
  let sqlContent = $state('')
  // 片段名→内容映射：extractParams 先展开 {{snippet:x}} 再提取参数，
  // 只出现在片段内容里的参数才能在编辑页看到输入框（与后端 sync 行为一致）
  let snippetMap = $state<Record<string, string>>({})
  let queryId = $state<number | null>(null)
  let queryName = $state('')
  let queryDesc = $state('')
  let moduleName = $state('')
  let categoryName = $state('')
  /** 0 单行/标量，1 多条（集合），与后端一致 */
  let isCollection = $state<SqlQueryCollectionFlag>(1)
  let editingName = $state(false)
  let editingDesc = $state(false)
  let nameInputRef = $state<HTMLInputElement | null>(null)
  let descTextareaRef = $state<HTMLTextAreaElement | null>(null)
  let results = $state<QueryResultRow[]>([])
  let isRunning = $state(false)
  let isSaving = $state(false)
  let savePromise: Promise<boolean> | null = null
  let executionTime = $state(0)
  let paramValues = $state<Record<string, string>>({})
  let pageIndex = $state(0)
  let pageSize = $state(10)
  let totalRows = $state(0)
  let resultFields = $state<SqlQueryResultFieldVo[]>([])
  /** 执行结果列名（adhoc 或未同步元数据时的兜底） */
  let previewFieldNames = $state<string[]>([])
  let vizConfig = $state<SqlQueryVizConfig>({ version: 1 })
  let vizOpen = $state(false)
  let aiDrawerOpen = $state(false)
  let aiInitialInput = $state('')
  // 右侧栏标签：元数据 / 片段
  let rightTab = $state<'schema' | 'snippet'>('schema')
  // Save-status state
  let savedSnapshot = $state<string | null>(null)
  let lastSavedAt = $state<string | null>(null)
  let nowTick = $state(Date.now())
  // Switch-query confirm state
  let switchDialogOpen = $state(false)
  let pendingQuery = $state<SqlQueryListVo | null>(null)
  let pendingIsNew = $state(false)
  let switchSaving = $state(false)
  // Data source selection
  let datasourceList = $state<DatasourceListVo[]>([])
  let datasourceLoading = $state(false)
  let schemaTables = $state<DatasourceTableSchemaVo[]>([])
  let previewTableName = $state('')
  let previewDrawerOpen = $state(false)
  let validateResult = $state<{ errors: string[]; warnings: string[] }>({
    errors: [],
    warnings: [],
  })

  function toCollectionFlag(v: unknown): SqlQueryCollectionFlag {
    return Number(v) === 0 ? 0 : 1
  }

  // Derived state
  let params = $derived(extractParams(sqlContent, snippetMap))
  /** 片段展开后的 SQL：供预览弹窗与 AI「附带当前 SQL」使用 */
  const expandedSql = $derived(expandSnippets(sqlContent, snippetMap))
  const expandedSqlFormatted = $derived.by(() =>
    expandedSql.trim() ? formatSql(expandedSql) : ''
  )
  /** 当前 SQL 引用的片段名（按出现顺序），供片段侧栏置顶与 AI 应用片段预选 */
  const usedSnippetNames = $derived(extractSnippetRefs(sqlContent))
  /** AI 附带 SQL：除展开结果外，附上原始 SQL 与片段定义，
   *  让 AI 能定位问题出在哪个片段（后端只在原文外层加 code fence，
   *  因此这里不能出现嵌套 ```） */
  const aiSqlContext = $derived.by(() => {
    const definedNames = usedSnippetNames.filter((n) => snippetMap[n])
    if (definedNames.length === 0) return expandedSql
    return [
      '【原始 SQL（含 {{snippet:…}} 片段引用）】',
      sqlContent.trim(),
      '',
      '【引用片段定义】',
      ...definedNames.map((n) => `■ 片段「${n}」\n${snippetMap[n].trim()}`),
      '',
      '【片段展开后的完整 SQL（实际执行）】',
      expandedSql.trim(),
      '',
      '说明：SQL 通过 {{snippet:片段名}} 引用以上片段。如果问题出在某个片段内，请指明片段名称，并单独给出该片段修正后的完整 SQL。',
    ].join('\n')
  })
  let previewSqlOpen = $state(false)

  // 编辑区/结果区分割比例（编辑区占工作区高度的百分比），持久化到 localStorage
  const SPLIT_KEY = 'sql-editor-split-pct'
  let splitPct = $state(loadSplit(SPLIT_KEY))
  let workbenchEl = $state<HTMLDivElement | null>(null)

  function setSplit(pct: number) {
    splitPct = clampSplit(pct)
    saveSplit(SPLIT_KEY, splitPct)
  }

  function handleSplitDragStart(e: MouseEvent) {
    if (!workbenchEl) return
    const height = workbenchEl.clientHeight
    if (height <= 0) return
    e.preventDefault()
    // 拖拽期间全局禁选中 + 行 resize 光标（body 级，指针移出工作区也生效）
    document.body.classList.add('select-none', 'cursor-row-resize')
    const startY = e.clientY
    const startPct = splitPct

    function onMove(ev: MouseEvent) {
      splitPct = clampSplit(startPct + ((ev.clientY - startY) / height) * 100)
    }
    function onUp() {
      document.body.classList.remove('select-none', 'cursor-row-resize')
      saveSplit(SPLIT_KEY, splitPct)
      window.removeEventListener('mousemove', onMove)
      window.removeEventListener('mouseup', onUp)
    }
    window.addEventListener('mousemove', onMove)
    window.addEventListener('mouseup', onUp)
  }

  function handleSplitKeydown(e: KeyboardEvent) {
    const step = 5
    if (e.key === 'ArrowUp') setSplit(splitPct - step)
    else if (e.key === 'ArrowDown') setSplit(splitPct + step)
    else if (e.key === 'Home') setSplit(SPLIT_MIN)
    else if (e.key === 'End') setSplit(SPLIT_MAX)
    else return
    e.preventDefault()
  }

  function takeSnapshot(): string {
    return buildSnapshot({
      queryName,
      queryDesc,
      sqlContent,
      isCollection,
      dataSourceId,
      module: moduleName,
      category: categoryName,
      vizConfig,
    })
  }

  const currentSnapshot = $derived(takeSnapshot())

  const categorySuggestions = $derived.by(() => {
    const match = moduleOptions.find((m) => m.name === moduleName)
    if (match) return match.categories
    return [...new Set(moduleOptions.flatMap((m) => m.categories))]
  })

  const isDirty = $derived.by(() => {
    if (savedSnapshot === null) {
      return (
        queryName.trim() !== '' ||
        queryDesc.trim() !== '' ||
        sqlContent.trim() !== ''
      )
    }
    return currentSnapshot !== savedSnapshot
  })

  const relativeSavedLabel = $derived(
    lastSavedAt ? formatRelativeTime(lastSavedAt, new Date(nowTick)) : ''
  )

  const showTag = $derived(
    (savedSnapshot !== null && lastSavedAt !== null) || isDirty
  )

  const tagText = $derived.by(() => {
    if (isDirty) {
      return relativeSavedLabel ? `未保存 · ${relativeSavedLabel}` : '未保存'
    }
    return relativeSavedLabel ? `已保存 · ${relativeSavedLabel}` : '已保存'
  })

  const datasourceItems = $derived(
    datasourceList.map((ds) => ({
      label: `${ds.name} (${ds.dbType})`,
      value: String(ds.dataSourceId),
    }))
  )

  const fieldsForViz = $derived(
    resultFields.length > 0
      ? resultFields.map((f) => ({
          fieldName: f.fieldName,
          commentLabel: f.commentLabel,
          fieldType: f.fieldType,
        }))
      : previewFieldNames.map((n) => ({ fieldName: n, fieldType: 'TEXT' }))
  )

  interface CodeEditorHandle {
    insertAtCursor: (text: string) => void
  }

  // Code editor ref
  let codeEditorRef: CodeEditorHandle | null = $state(null)

  let nowTickIntervalId: ReturnType<typeof setInterval> | null = null

  onMount(() => {
    nowTickIntervalId = setInterval(() => {
      nowTick = Date.now()
    }, 30_000)
  })

  onDestroy(() => {
    if (nowTickIntervalId) clearInterval(nowTickIntervalId)
  })

  function handleBeforeUnload(e: BeforeUnloadEvent) {
    if (!isDirty) return
    e.preventDefault()
    e.returnValue = ''
  }

  onMount(() => {
    window.addEventListener('beforeunload', handleBeforeUnload)
  })

  onDestroy(() => {
    window.removeEventListener('beforeunload', handleBeforeUnload)
  })

  function handleClose() {
    if (isDirty && !window.confirm('有未保存的更改,确定离开吗?')) return
    queryState.closeEditor()
  }

  // Load (or reset) the editor on each open. The drawer is mounted once and
  // reused, so internal state must be explicitly (re)initialised every time it
  // opens — otherwise "添加查询" would inherit the previous session's content.
  let wasEditorOpen = false
  $effect(() => {
    const open = queryState.editorOpen
    if (open && !wasEditorOpen) {
      const eId = queryState.editingQueryId
      const cId = queryState.copyFromId
      loadSnippetMap()
      if (eId != null) {
        loadQuery(eId)
      } else if (cId != null) {
        loadQueryForCopy(cId)
      } else {
        resetEditor()
      }
    }
    wasEditorOpen = open
  })

  async function loadSnippetMap() {
    try {
      const res = await sqlSnippetsApi.list()
      const map: Record<string, string> = {}
      for (const s of res.data ?? []) {
        map[s.sqlSnippetName] = s.sqlSnippetContent
      }
      snippetMap = map
    } catch {
      // 加载失败不阻塞编辑器，仅片段内参数暂不可见
    }
  }

  // Load dynamic components and datasources
  onMount(async () => {
    // Dynamic import of components (client-side only)
    const [editorMod, tableMod, sidebarMod, schemaMod, selectorMod, drawerMod] =
      await Promise.all([
        import('$lib/components/sql-editor/SqlCodeEditor.svelte'),
        import('$lib/components/sql-editor/QueryResultsTable.svelte'),
        import('$lib/components/sql-editor/SnippetSidebar.svelte'),
        import('$lib/components/sql-editor/SchemaSidebar.svelte'),
        import('$lib/components/sql-editor/QuerySelector.svelte'),
        import('$lib/components/sql-editor/TableDataDrawer.svelte'),
      ])
    SqlCodeEditor = editorMod.default
    QueryResultsTable = tableMod.default
    SnippetSidebar = sidebarMod.default
    SchemaSidebar = schemaMod.default
    QuerySelector = selectorMod.default
    TableDataDrawer = drawerMod.default

    // Load datasources for dropdown
    datasourceLoading = true
    try {
      const res = await datasourcesApi.list()
      datasourceList = res.data ?? []
    } catch (e) {
      if (!isApiClientNotifiedError(e)) {
        toast.error('加载数据源列表失败')
      }
    } finally {
      datasourceLoading = false
    }
  })

  function applyQueryData(data: SqlQueryDetailVo) {
    queryName = data.sqlQueryName
    queryDesc = data.sqlQueryDesc ?? ''
    sqlContent = data.sqlQueryContent
    paramValues = {}
    isCollection = toCollectionFlag(data.isCollection)
    dataSourceId = data.dataSourceId ?? 0
    moduleName = data.module ?? ''
    categoryName = data.category ?? ''
    resultFields = data.resultFields ?? []
    vizConfig = normalizeVizConfig(
      data.vizConfig,
      isCollection,
      resultFields.map((f) => ({
        fieldName: f.fieldName,
        commentLabel: f.commentLabel,
        fieldType: f.fieldType,
      }))
    )
  }

  // Load query by ID
  async function loadQuery(id: number) {
    try {
      const query = await sqlQueriesApi.getById(id)
      if (query.data) {
        applyQueryData(query.data)
        queryId = query.data.sqlQueryId
        savedSnapshot = takeSnapshot()
        lastSavedAt = query.data.lastModifiedDate ?? null
      }
    } catch (e) {
      if (!isApiClientNotifiedError(e)) {
        toast.error('加载查询失败')
      }
    }
  }

  async function loadQueryForCopy(id: number) {
    try {
      const query = await sqlQueriesApi.getById(id)
      if (query.data) {
        applyQueryData(query.data)
        queryName = queryName + '_副本'
        queryId = null
        savedSnapshot = takeSnapshot()
      }
    } catch (e) {
      if (!isApiClientNotifiedError(e)) {
        toast.error('加载查询失败')
      }
    }
  }

  function resetEditor() {
    editingName = false
    editingDesc = false
    queryId = null
    queryName = ''
    queryDesc = ''
    sqlContent = ''
    paramValues = {}
    isCollection = 1
    dataSourceId = 0
    moduleName = ''
    categoryName = ''
    resultFields = []
    previewFieldNames = []
    vizConfig = normalizeVizConfig(null, 1, [])
    results = []
    savedSnapshot = null
    lastSavedAt = null
  }

  async function doSwitchQuery(query: SqlQueryListVo | null) {
    editingName = false
    editingDesc = false
    if (query) {
      await loadQuery(query.sqlQueryId)
    } else {
      resetEditor()
    }
  }

  async function handleQuerySelect(query: SqlQueryListVo | null) {
    if (isDirty) {
      pendingQuery = query
      pendingIsNew = query === null
      switchDialogOpen = true
      return
    }
    await doSwitchQuery(query)
  }

  function handleSwitchCancel() {
    switchDialogOpen = false
    pendingQuery = null
    pendingIsNew = false
    // 触发 QuerySelector 重新同步 value 显示
    queryId = queryId
  }

  async function handleSwitchDiscard() {
    const target = pendingIsNew ? null : pendingQuery
    switchDialogOpen = false
    pendingQuery = null
    pendingIsNew = false
    await doSwitchQuery(target)
  }

  async function handleSwitchSaveAndSwitch() {
    const target = pendingIsNew ? null : pendingQuery
    switchSaving = true
    try {
      const ok = await performSave()
      if (!ok) return // 让 dialog 保持开启,等用户修正或取消
      switchDialogOpen = false
      pendingQuery = null
      pendingIsNew = false
      await doSwitchQuery(target)
    } finally {
      switchSaving = false
    }
  }

  $effect(() => {
    if (editingName) {
      void tick().then(() => nameInputRef?.focus())
    }
  })

  $effect(() => {
    if (editingDesc) {
      void tick().then(() => descTextareaRef?.focus())
    }
  })

  function finishEditingName() {
    editingName = false
  }

  function finishEditingDesc() {
    editingDesc = false
  }

  // Handle snippet insertion
  function handleSnippetInsert(snippetRef: string) {
    if (codeEditorRef?.insertAtCursor) {
      codeEditorRef.insertAtCursor(snippetRef)
    }
  }

  /** AI 助手里「应用到片段」保存成功后，同步刷新片段映射与侧栏列表 */
  function handleSnippetUpdatedFromAi() {
    void loadSnippetMap()
    snippetSidebarRef?.reload()
  }

  function handleSchemaLoad(tables: DatasourceTableSchemaVo[]) {
    schemaTables = tables
  }

  function handlePreviewTable(name: string) {
    previewTableName = name
    previewDrawerOpen = true
  }

  function handleValidate(result: { errors: string[]; warnings: string[] }) {
    validateResult = result
  }

  // Run SQL query — 始终使用编辑器中的 SQL 内容（adhoc），
  // 避免 queryId 存在时后端用数据库里保存的旧 SQL 执行。
  async function handleRun(pi = pageIndex, ps = pageSize) {
    if (!sqlContent.trim()) {
      toast.error('请输入 SQL 查询')
      return
    }

    const autoSaved = await autoSaveIfPossible()

    isRunning = true
    const startTime = performance.now()

    try {
      const response = await sqlQueriesApi.runAdhoc({
        sqlQueryContent: sqlContent,
        dataSourceId: dataSourceId,
        params: paramValues,
        pageNo: pi + 1,
        pageSize: ps,
      })
      results = response.data?.rows ?? []
      totalRows = response.data?.total ?? results.length
      previewFieldNames =
        response.data?.columns?.map((c) => c.name).filter(Boolean) ?? []
      executionTime = performance.now() - startTime

      // 用实际执行结果的列信息来驱动 vizConfig，
      // 同时保留已有 resultFields 中匹配的 commentLabel 等元数据。
      const columnSet = new Set(previewFieldNames)
      const existingMap = new Map(resultFields.map((f) => [f.fieldName, f]))
      // 只保留在本次结果中仍存在的字段
      resultFields = resultFields.filter((f) => columnSet.has(f.fieldName))
      const fv = previewFieldNames.map((n) => {
        const existing = existingMap.get(n)
        return existing
          ? {
              fieldName: existing.fieldName,
              commentLabel: existing.commentLabel,
              fieldType: existing.fieldType,
            }
          : { fieldName: n, fieldType: 'TEXT' }
      })
      vizConfig = normalizeVizConfig(vizConfig, isCollection, fv)
      toast.success(`查询执行成功，共 ${totalRows} 行`)

      if (autoSaved) {
        savedSnapshot = takeSnapshot()
      }
    } catch (e) {
      if (!isApiClientNotifiedError(e)) {
        toast.error(e instanceof Error ? e.message : '执行查询失败')
      }
    } finally {
      isRunning = false
    }
  }

  // Manual save (with validation toasts and success toast).
  // Returns true on success, false on validation failure or API error.
  async function performSave(): Promise<boolean> {
    if (savePromise) return savePromise

    if (!sqlContent.trim()) {
      toast.error('请输入 SQL 查询')
      return false
    }

    if (!queryName.trim()) {
      toast.error('请输入查询名称')
      return false
    }

    savePromise = (async () => {
      isSaving = true

      try {
        if (queryId) {
          await sqlQueriesApi.update({
            sqlQueryId: queryId,
            sqlQueryName: queryName,
            sqlQueryDesc: queryDesc.trim() || undefined,
            sqlQueryContent: sqlContent,
            dataSourceId,
            isCollection,
            vizConfig,
            module: moduleName.trim() || undefined,
            category: categoryName.trim() || undefined,
          })
          toast.success('查询更新成功')
          lastSavedAt = new Date().toISOString()
          nowTick = Date.now()
          savedSnapshot = takeSnapshot()
          return true
        } else {
          const newId = await sqlQueriesApi.create({
            sqlQueryName: queryName,
            sqlQueryDesc: queryDesc.trim() || undefined,
            sqlQueryContent: sqlContent,
            dataSourceId,
            isCollection,
            vizConfig,
            module: moduleName.trim() || undefined,
            category: categoryName.trim() || undefined,
          })
          toast.success('查询创建成功')
          queryId = newId.data!
          lastSavedAt = new Date().toISOString()
          nowTick = Date.now()
          savedSnapshot = takeSnapshot()
          return true
        }
      } catch (e) {
        if (!isApiClientNotifiedError(e)) {
          toast.error(e instanceof Error ? e.message : '保存查询失败')
        }
        return false
      } finally {
        isSaving = false
        savePromise = null
      }
    })()

    return savePromise
  }

  async function handleSave() {
    await performSave()
  }

  // Silent save used by format/run. Skips toasts and never blocks the caller.
  // Returns true if the snapshot is now clean (either was clean already or
  // the save succeeded), false otherwise.
  async function autoSaveIfPossible(): Promise<boolean> {
    if (!isDirty) return true
    if (!queryName.trim() || !sqlContent.trim() || dataSourceId === 0) {
      return false
    }
    if (savePromise) return savePromise

    savePromise = (async () => {
      isSaving = true
      try {
        if (queryId) {
          await sqlQueriesApi.update({
            sqlQueryId: queryId,
            sqlQueryName: queryName,
            sqlQueryDesc: queryDesc.trim() || undefined,
            sqlQueryContent: sqlContent,
            dataSourceId,
            isCollection,
            vizConfig,
            module: moduleName.trim() || undefined,
            category: categoryName.trim() || undefined,
          })
          lastSavedAt = new Date().toISOString()
          nowTick = Date.now()
          savedSnapshot = takeSnapshot()
          return true
        } else {
          const newId = await sqlQueriesApi.create({
            sqlQueryName: queryName,
            sqlQueryDesc: queryDesc.trim() || undefined,
            sqlQueryContent: sqlContent,
            dataSourceId,
            isCollection,
            vizConfig,
            module: moduleName.trim() || undefined,
            category: categoryName.trim() || undefined,
          })
          queryId = newId.data!
          lastSavedAt = new Date().toISOString()
          nowTick = Date.now()
          savedSnapshot = takeSnapshot()
          return true
        }
      } catch (e) {
        if (!isApiClientNotifiedError(e)) {
          toast.error(e instanceof Error ? e.message : '保存查询失败')
        }
        return false
      } finally {
        isSaving = false
        savePromise = null
      }
    })()

    return savePromise
  }

  async function persistVizConfigFromDrawer() {
    const id = queryId
    if (id == null) return
    try {
      await sqlQueriesApi.saveVizConfig(id, { vizConfig })
      toast.success('可视化配置已保存')
    } catch (e) {
      if (!isApiClientNotifiedError(e)) {
        toast.error(e instanceof Error ? e.message : '保存可视化配置失败')
      }
      throw e
    }
  }

  // Format SQL content
  async function handleFormat() {
    if (!sqlContent.trim()) {
      toast.error('请输入 SQL 查询')
      return
    }
    try {
      sqlContent = formatSql(sqlContent)
      toast.success('SQL 已格式化')
      await autoSaveIfPossible()
    } catch (e) {
      toast.error(
        e instanceof Error ? `格式化失败: ${e.message}` : '格式化失败'
      )
    }
  }

  // Handle keyboard shortcut
  function handleKeyDown(e: KeyboardEvent) {
    if (e.ctrlKey && e.key === 'Enter') {
      e.preventDefault()
      handleRun()
    } else if (e.ctrlKey && e.shiftKey && (e.key === 'F' || e.key === 'f')) {
      e.preventDefault()
      handleFormat()
    }
  }
</script>

<svelte:window onkeydown={handleKeyDown} />

<Sheet.Root
  bind:open={
    () => queryState.editorOpen,
    (v) => {
      // Route every close attempt (ESC, overlay, built-in close button) through
      // the dirty guard. Declining the confirm leaves editorOpen untouched, so
      // the controlled binding keeps the sheet open instead of bypassing it.
      if (v) queryState.editorOpen = true
      else handleClose()
    }
  }
>
  <Sheet.Portal>
    <Sheet.Overlay />
    <Sheet.Content
      class="flex !h-full !w-full !max-w-none flex-col border-l p-0"
      showClose={false}
    >
      <!-- Left drag handle (4px) -->
      <div
        role="separator"
        class="hover:bg-primary/20 absolute top-0 bottom-0 left-0 z-10 w-1 cursor-col-resize"
      ></div>

      <!-- header 承载全部主操作：在抽屉体滚动区之外，任何视口下都可达 -->
      <Sheet.Header
        class="flex flex-row flex-wrap items-center gap-2 border-b px-4 py-2"
      >
        <!-- 标题组兼作 spacer：吃掉剩余宽度，Badge 紧贴名称 -->
        <div class="flex min-w-0 flex-1 items-center gap-2">
          <Sheet.Title class="min-w-0 text-base font-semibold">
            {#if editingName}
              <Input
                bind:ref={nameInputRef}
                bind:value={queryName}
                placeholder="输入查询名称"
                class="h-8 w-64 max-w-full text-base font-semibold"
                onblur={finishEditingName}
                onkeydown={(e) => {
                  if (e.key === 'Enter') {
                    e.preventDefault()
                    finishEditingName()
                  }
                }}
              />
            {:else}
              <button
                type="button"
                class="hover:bg-accent/50 block max-w-full truncate rounded-md px-1 py-0.5 text-left"
                class:text-muted-foreground={!queryName.trim()}
                title="点击编辑名称"
                onclick={() => (editingName = true)}
              >
                {queryName.trim() || '点击输入查询名称'}
              </button>
            {/if}
          </Sheet.Title>
          {#if showTag}
            {#if isDirty}
              <Badge variant="destructive" class="shrink-0">{tagText}</Badge>
            {:else}
              <Badge
                variant="secondary"
                class="shrink-0 bg-emerald-100 text-emerald-700 dark:bg-emerald-900/30 dark:text-emerald-300"
              >
                {tagText}
              </Badge>
            {/if}
          {/if}
        </div>
        <div class="flex shrink-0 flex-wrap items-center gap-2">
          {#if QuerySelector}
            <QuerySelector value={queryId} onselect={handleQuerySelect} />
          {/if}
          <SelectDropdown
            value={String(dataSourceId)}
            onValueChange={(v) => {
              dataSourceId = Number(v)
            }}
            items={datasourceItems}
            isPending={datasourceLoading}
            placeholder="选择数据源"
            class="w-48"
          />
          <Button variant="outline" onclick={handleFormat} title="Ctrl+Shift+F">
            <Wand2 class="mr-2 h-4 w-4" />
            格式化
          </Button>
          <Button variant="outline" onclick={handleSave} disabled={isSaving}>
            {#if isSaving}
              <LoaderCircle class="mr-2 h-4 w-4 animate-spin" />
            {:else}
              <Save class="mr-2 h-4 w-4" />
            {/if}
            保存
          </Button>
          <Button onclick={() => handleRun()} disabled={isSaving || isRunning}>
            {#if isRunning}
              <LoaderCircle class="mr-2 h-4 w-4 animate-spin" />
            {:else}
              <Play class="mr-2 h-4 w-4" />
            {/if}
            执行 (Ctrl+Enter)
          </Button>
        </div>
        <Button
          variant="ghost"
          size="icon"
          onclick={handleClose}
          title="关闭编辑器"
        >
          <X class="h-4 w-4" />
        </Button>
      </Sheet.Header>

      <!-- 抽屉体是唯一允许"内容超出就滚"的层；主操作在 header，不受滚动影响 -->
      <div
        class="flex min-h-0 min-w-0 flex-1 flex-col gap-3 overflow-y-auto p-4"
      >
        <!-- 元信息行：描述 | 功能模块 | 分类 | 返回多条（名称已上移到 header） -->
        <div class="flex flex-wrap items-center gap-2">
          {#if editingDesc}
            <textarea
              bind:this={descTextareaRef}
              bind:value={queryDesc}
              placeholder="简要说明该查询的用途（可选）"
              rows="2"
              class="border-input bg-background ring-offset-background placeholder:text-muted-foreground focus-visible:ring-ring w-full basis-full rounded-md border px-3 py-2 text-sm focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:outline-none"
              onblur={finishEditingDesc}
            ></textarea>
          {:else}
            <button
              type="button"
              class="hover:bg-accent/50 min-w-40 flex-1 truncate rounded-md px-1 py-0.5 text-left text-sm"
              title="点击编辑描述"
              onclick={() => (editingDesc = true)}
            >
              <span class:text-muted-foreground={!queryDesc.trim()}>
                {queryDesc.trim() || '点击添加描述（可选）'}
              </span>
            </button>
          {/if}
          <label for="sql-editor-module" class="sr-only">功能模块</label>
          <Input
            id="sql-editor-module"
            bind:value={moduleName}
            list="sql-editor-module-list"
            placeholder="功能模块（可选）"
            class="h-8 w-40"
          />
          <label for="sql-editor-category" class="sr-only">分类</label>
          <Input
            id="sql-editor-category"
            bind:value={categoryName}
            list="sql-editor-category-list"
            placeholder="分类（可选）"
            class="h-8 w-40"
          />
          <label
            class="text-muted-foreground flex shrink-0 cursor-pointer items-center gap-2 text-sm"
          >
            <Checkbox
              checked={isCollection === 1}
              onCheckedChange={(v) => {
                const next: SqlQueryCollectionFlag = v === true ? 1 : 0
                isCollection = next
                vizConfig = normalizeVizConfig(vizConfig, next, fieldsForViz)
              }}
            />
            <span class="text-foreground font-medium">返回多条</span>
          </label>
          <datalist id="sql-editor-module-list">
            {#each moduleOptions as opt (opt.name)}
              <option value={opt.name}></option>
            {/each}
          </datalist>
          <datalist id="sql-editor-category-list">
            {#each categorySuggestions as c (c)}
              <option value={c}></option>
            {/each}
          </datalist>
        </div>

        <!-- Parameter inputs -->
        {#if params.length > 0}
          <div class="rounded-md border p-4">
            <h3 class="mb-2 text-sm font-medium">查询参数</h3>
            <div class="flex flex-wrap gap-4">
              {#each params as param}
                <div class="flex items-center gap-2">
                  <Label for={`param-${param}`} class="text-sm">{param}</Label>
                  <Input
                    id={`param-${param}`}
                    bind:value={paramValues[param]}
                    placeholder={`输入 ${param} 的值...`}
                    class="w-48"
                  />
                </div>
              {/each}
            </div>
          </div>
        {/if}

        <!-- 工作区：编辑区（含校验信息）+ 分割条 + 结果区。
             min-h-0 必须：否则编辑区 50% 的百分比 flex-basis 在工作区高度被内容
             决定时回退为 content，CM 文档高会把工作区撑穿；放不下时子项按 min-h
             地板保持最小高度，可见溢出并入抽屉体滚动区（设计内兜底） -->
        <div bind:this={workbenchEl} class="flex min-h-0 flex-1 flex-col">
          <!-- 编辑区：高度 = 工作区的百分比，下限 180px -->
          <div
            class="flex min-h-[180px] flex-col"
            style="flex: 0 0 {splitPct}%"
          >
            <div class="flex min-h-0 flex-1 gap-2 sm:gap-4">
              <div class="flex shrink-0 flex-col gap-2 pt-1">
                <Button
                  variant="outline"
                  size="sm"
                  class="h-auto flex-col gap-1 px-2 py-2"
                  type="button"
                  onclick={() => (previewSqlOpen = true)}
                  disabled={!expandedSql.trim()}
                  title="预览片段展开后的 SQL"
                >
                  <Eye class="size-4 shrink-0" />
                  <span class="text-[10px] leading-tight font-medium">预览</span
                  >
                </Button>
                <Button
                  variant="outline"
                  size="sm"
                  class="h-auto flex-col gap-1 px-2 py-2"
                  type="button"
                  onclick={() => (vizOpen = true)}
                  title="可视化配置"
                >
                  <Settings2 class="size-4 shrink-0" />
                  <span class="text-[10px] leading-tight font-medium"
                    >可视化</span
                  >
                </Button>
                <Button
                  variant="outline"
                  size="sm"
                  class="h-auto flex-col gap-1 px-2 py-2"
                  type="button"
                  onclick={() => (aiDrawerOpen = true)}
                  disabled={queryId == null}
                  title={queryId == null ? '请先保存 SQL Query' : 'AI 助手'}
                >
                  <Bot class="size-4 shrink-0" />
                  <span class="text-[10px] leading-tight font-medium">AI</span>
                </Button>
              </div>
              <!-- SQL Editor -->
              <div class="flex min-w-0 flex-1 flex-col">
                {#if SqlCodeEditor}
                  <SqlCodeEditor
                    bind:value={sqlContent}
                    bind:this={codeEditorRef}
                    onrun={handleRun}
                    schema={schemaTables}
                    {dataSourceId}
                    onvalidate={handleValidate}
                    class="flex-1"
                  />
                {:else}
                  <div
                    class="bg-muted flex flex-1 items-center justify-center rounded-md border"
                  >
                    <LoaderCircle
                      class="text-muted-foreground h-8 w-8 animate-spin"
                    />
                  </div>
                {/if}
              </div>

              <!-- Right sidebar: 元数据 / 片段 tabs -->
              <div class="hidden w-64 shrink-0 lg:flex lg:flex-col">
                <div class="flex shrink-0 gap-1 border-b border-l p-1">
                  <button
                    type="button"
                    class="flex-1 rounded-sm px-2 py-1.5 text-xs font-medium transition-colors {rightTab ===
                    'schema'
                      ? 'bg-accent text-accent-foreground'
                      : 'text-muted-foreground hover:bg-accent/50'}"
                    onclick={() => (rightTab = 'schema')}
                  >
                    元数据
                  </button>
                  <button
                    type="button"
                    class="flex-1 rounded-sm px-2 py-1.5 text-xs font-medium transition-colors {rightTab ===
                    'snippet'
                      ? 'bg-accent text-accent-foreground'
                      : 'text-muted-foreground hover:bg-accent/50'}"
                    onclick={() => (rightTab = 'snippet')}
                  >
                    片段
                  </button>
                </div>
                <div class="min-h-0 flex-1">
                  {#if rightTab === 'schema'}
                    {#if SchemaSidebar}
                      <SchemaSidebar
                        {dataSourceId}
                        oninsert={handleSnippetInsert}
                        onpreviewtable={handlePreviewTable}
                        onschemaload={handleSchemaLoad}
                      />
                    {:else}
                      <div
                        class="bg-muted flex h-full items-center justify-center border-l"
                      >
                        <LoaderCircle
                          class="text-muted-foreground h-6 w-6 animate-spin"
                        />
                      </div>
                    {/if}
                  {:else if SnippetSidebar}
                    <SnippetSidebar
                      bind:this={snippetSidebarRef}
                      oninsert={handleSnippetInsert}
                      usedNames={usedSnippetNames}
                    />
                  {:else}
                    <div
                      class="bg-muted flex h-full items-center justify-center border-l"
                    >
                      <LoaderCircle
                        class="text-muted-foreground h-6 w-6 animate-spin"
                      />
                    </div>
                  {/if}
                </div>
              </div>
            </div>

            {#if validateResult.errors.length > 0 || validateResult.warnings.length > 0}
              <div class="mt-2 shrink-0 space-y-1 rounded-md border px-3 py-2">
                {#each validateResult.errors as msg}
                  <div
                    class="flex items-center gap-2 text-sm text-red-600 dark:text-red-400"
                  >
                    <span class="shrink-0 font-medium">Error</span>
                    <span class="flex-1">{msg}</span>
                    <button
                      type="button"
                      class="text-muted-foreground hover:text-primary shrink-0 text-xs underline"
                      onclick={() => {
                        aiInitialInput = `SQL执行出错：\n${msg}\n\n请帮我分析这个错误并提供解决方案。`
                        aiDrawerOpen = true
                      }}
                    >
                      问AI
                    </button>
                  </div>
                {/each}
                {#each validateResult.warnings as msg}
                  <div
                    class="flex items-center gap-2 text-sm text-amber-600 dark:text-amber-400"
                  >
                    <span class="shrink-0 font-medium">Warn</span>
                    <span class="flex-1">{msg}</span>
                    <button
                      type="button"
                      class="text-muted-foreground hover:text-primary shrink-0 text-xs underline"
                      onclick={() => {
                        aiInitialInput = `SQL警告：\n${msg}\n\n请帮我分析这个问题。`
                        aiDrawerOpen = true
                      }}
                    >
                      问AI
                    </button>
                  </div>
                {/each}
              </div>
            {/if}
          </div>

          <!-- 分割条：拖拽 / 双击复位 / ↑↓ 5% / Home End 到边界 -->
          <!-- 可聚焦的 separator 按 ARIA 属交互组件，Svelte 检查器不识别，忽略误报 -->
          <!-- svelte-ignore a11y_no_noninteractive_element_interactions, a11y_no_noninteractive_tabindex -->
          <div
            role="separator"
            aria-orientation="horizontal"
            aria-label="调整编辑区与结果区高度"
            aria-valuemin={SPLIT_MIN}
            aria-valuemax={SPLIT_MAX}
            aria-valuenow={splitPct}
            tabindex="0"
            class="bg-border hover:bg-primary/40 active:bg-primary/60 focus-visible:bg-primary/40 my-1 h-1.5 shrink-0 cursor-row-resize rounded-full outline-none"
            title="拖动调整高度，双击复位"
            onmousedown={handleSplitDragStart}
            ondblclick={() => setSplit(SPLIT_DEFAULT)}
            onkeydown={handleSplitKeydown}
          ></div>

          <!-- 结果区：吃掉剩余空间，内部滚动 -->
          <div class="min-h-[120px] w-full min-w-0 flex-1 overflow-auto">
            {#if isCollection === 0}
              <div class="space-y-2">
                {#if isRunning}
                  <div
                    class="bg-muted flex h-40 items-center justify-center rounded-md border"
                  >
                    <LoaderCircle
                      class="text-muted-foreground h-8 w-8 animate-spin"
                    />
                  </div>
                {:else if results.length === 0}
                  <p class="text-muted-foreground text-sm">暂无查询结果。</p>
                {:else}
                  <div
                    class="text-muted-foreground flex items-center justify-between text-sm"
                  >
                    <span>{totalRows} 行</span>
                    <span>{executionTime} ms</span>
                  </div>
                  {@const sf = vizConfig.scalarField}
                  {@const v =
                    sf != null && sf !== '' && results[0]
                      ? results[0][sf]
                      : undefined}
                  <Card.Root class="max-w-md">
                    <Card.Header class="pb-2">
                      <Card.Title class="text-base">
                        {sf
                          ? resultFields.find((r) => r.fieldName === sf)
                              ?.commentLabel || sf
                          : '值'}
                      </Card.Title>
                      <Card.Description class="font-mono text-xs">
                        {sf ?? '未选择字段'}
                      </Card.Description>
                    </Card.Header>
                    <Card.Content>
                      <p
                        class="text-3xl font-semibold tracking-tight tabular-nums"
                      >
                        {v === null || v === undefined ? '—' : String(v)}
                      </p>
                    </Card.Content>
                  </Card.Root>
                {/if}
              </div>
            {:else if QueryResultsTable}
              <QueryResultsTable
                data={results}
                loading={isRunning}
                rowCount={totalRows}
                {executionTime}
                vizTable={vizConfig.table}
                {resultFields}
                columnOrder={previewFieldNames}
                serverTotal={totalRows}
                onPageChange={(pi: number, ps: number) => handleRun(pi, ps)}
              />
            {/if}
          </div>
        </div>
      </div>

      <!-- 预览 Dialog 必须嵌套在 Sheet.Content 内部：bits-ui 按组件嵌套
           关系管理弹层层序，放在外层（兄弟节点）时 portal 会被插入到
           全屏 Sheet 之前，Dialog 被盖住表现为"点击无反应" -->
      <Dialog.Root bind:open={previewSqlOpen}>
        <Dialog.Content class="flex max-h-[80vh] flex-col sm:max-w-3xl">
          <Dialog.Header>
            <Dialog.Title>SQL 预览</Dialog.Title>
            <Dialog.Description>
              仅展开 <code>{'{{snippet:…}}'}</code>
              引用，便于查看最终执行的 SQL；参数占位符保持原样。
            </Dialog.Description>
          </Dialog.Header>
          <div class="min-h-0 flex-1 overflow-auto">
            {#if expandedSqlFormatted}
              <pre
                class="bg-muted/50 text-foreground/90 max-h-[55vh] overflow-auto rounded-md border p-3 font-mono text-xs leading-relaxed">{expandedSqlFormatted}</pre>
            {:else}
              <p class="text-muted-foreground p-4 text-sm">当前 SQL 为空</p>
            {/if}
          </div>
        </Dialog.Content>
      </Dialog.Root>
    </Sheet.Content>
  </Sheet.Portal>
</Sheet.Root>

<SqlVisualizationDrawer
  bind:open={vizOpen}
  {isCollection}
  {resultFields}
  {previewFieldNames}
  bind:vizConfig
  onVizConfigPersist={queryId != null ? persistVizConfigFromDrawer : undefined}
/>

{#if queryId != null}
  <AiChatDrawer
    open={aiDrawerOpen}
    sqlQueryId={queryId}
    currentSql={aiSqlContext}
    {usedSnippetNames}
    initialInput={aiInitialInput}
    onSnippetUpdated={handleSnippetUpdatedFromAi}
    onOpenChange={(o) => {
      aiDrawerOpen = o
      if (!o) aiInitialInput = ''
    }}
  />
{/if}

{#if TableDataDrawer}
  <TableDataDrawer
    bind:open={previewDrawerOpen}
    tableName={previewTableName}
    {dataSourceId}
  />
{/if}

<AlertDialog.Root
  open={switchDialogOpen}
  onOpenChange={(o) => {
    if (!o) handleSwitchCancel()
  }}
>
  <AlertDialog.Content class="max-w-md">
    <AlertDialog.Header class="text-start">
      <AlertDialog.Title>当前查询有未保存的更改</AlertDialog.Title>
      <AlertDialog.Description>
        {#if pendingIsNew}
          切换到「新建查询」会丢失这些更改。
        {:else}
          切换到「{pendingQuery?.sqlQueryName ?? ''}」会丢失这些更改。
        {/if}
      </AlertDialog.Description>
    </AlertDialog.Header>

    <AlertDialog.Footer>
      <Button
        variant="outline"
        onclick={handleSwitchCancel}
        disabled={switchSaving}
      >
        取消
      </Button>
      <Button
        variant="destructive"
        onclick={handleSwitchDiscard}
        disabled={switchSaving}
      >
        丢弃更改
      </Button>
      <Button onclick={handleSwitchSaveAndSwitch} disabled={switchSaving}>
        {#if switchSaving}
          <LoaderCircle class="mr-2 h-4 w-4 animate-spin" />
        {/if}
        保存并切换
      </Button>
    </AlertDialog.Footer>
  </AlertDialog.Content>
</AlertDialog.Root>

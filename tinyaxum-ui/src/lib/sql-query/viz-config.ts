import type {
  SqlQueryCollectionFlag,
  SqlQueryVizChart,
  SqlQueryVizChartSeries,
  SqlQueryVizConfig,
  SqlQueryVizTableColumn,
} from '$lib/api/types.js'

export type VizFieldMeta = {
  fieldName: string
  commentLabel?: string
  fieldType?: string
}

export function defaultVizConfig(
  isCollection: SqlQueryCollectionFlag,
  fields: VizFieldMeta[]
): SqlQueryVizConfig {
  if (isCollection === 0) {
    return {
      version: 1,
      scalarField: fields[0]?.fieldName,
    }
  }
  return {
    version: 1,
    kind: 'table',
    table: {
      showRowIndex: false,
      columns: fields.map((f) => ({
        fieldName: f.fieldName,
        title: f.commentLabel || f.fieldName,
        align: 'left' as const,
        visible: true,
      })),
    },
  }
}

function defaultChart(fields: VizFieldMeta[]): SqlQueryVizChart | undefined {
  if (fields.length === 0) return undefined
  const isDateLike = (t: string | undefined) => {
    if (!t) return false
    const u = t.toUpperCase()
    return u.includes('DATE') || u.includes('TIME')
  }
  const xField =
    fields.find((f) => isDateLike(f.fieldType))?.fieldName ??
    fields[0].fieldName
  const numericFields = fields.filter(
    (f) => f.fieldName !== xField && isNumericFieldType(f.fieldType)
  )
  const seriesSource =
    numericFields.length > 0
      ? numericFields
      : fields.filter((f) => f.fieldName !== xField).slice(0, 1)
  const series: SqlQueryVizChartSeries[] = seriesSource.map((f) => ({
    fieldName: f.fieldName,
    type: 'line',
    label: f.commentLabel || f.fieldName,
  }))
  if (series.length === 0) return undefined
  return { xField, series }
}

/** 与当前字段列表对齐：保留顺序与自定义项，补齐新字段。 */
export function normalizeVizConfig(
  raw: SqlQueryVizConfig | null | undefined,
  isCollection: SqlQueryCollectionFlag,
  fields: VizFieldMeta[]
): SqlQueryVizConfig {
  const base = defaultVizConfig(isCollection, fields)
  if (!raw || raw.version !== 1) {
    return base
  }

  if (isCollection === 0) {
    const sf = raw.scalarField
    const ok = sf && fields.some((f) => f.fieldName === sf)
    return {
      version: 1,
      scalarField: ok ? sf : base.scalarField,
    }
  }

  const kind: 'table' | 'chart' = raw.kind === 'chart' ? 'chart' : 'table'

  if (kind === 'table') {
    const prev = raw.table?.columns ?? []
    const ordered: SqlQueryVizTableColumn[] = []
    const seen = new Set<string>()

    for (const c of prev) {
      if (!fields.some((f) => f.fieldName === c.fieldName)) continue
      const meta = fields.find((f) => f.fieldName === c.fieldName)
      ordered.push({
        fieldName: c.fieldName,
        title: c.title ?? meta?.commentLabel ?? c.fieldName,
        align: c.align === 'center' || c.align === 'right' ? c.align : 'left',
        visible: c.visible !== false,
      })
      seen.add(c.fieldName)
    }

    for (const f of fields) {
      if (seen.has(f.fieldName)) continue
      ordered.push({
        fieldName: f.fieldName,
        title: f.commentLabel || f.fieldName,
        align: 'left',
        visible: true,
      })
    }

    return {
      version: 1,
      kind: 'table',
      table: {
        showRowIndex: raw.table?.showRowIndex === true,
        columns: ordered,
      },
    }
  }

  const fieldByName = new Map(fields.map((f) => [f.fieldName, f] as const))
  const xField =
    raw.chart?.xField && fieldByName.has(raw.chart.xField)
      ? raw.chart.xField
      : (defaultChart(fields)?.xField ?? fields[0]?.fieldName ?? '')
  const series: SqlQueryVizChartSeries[] = (raw.chart?.series ?? [])
    .filter((s) => fieldByName.has(s.fieldName) && s.fieldName !== xField)
    .map((s) => ({
      fieldName: s.fieldName,
      type: s.type === 'area' ? 'area' : 'line',
      label: s.label ?? fieldByName.get(s.fieldName)?.commentLabel,
    }))
  const finalSeries =
    series.length > 0 ? series : (defaultChart(fields)?.series ?? [])
  return {
    version: 1,
    kind: 'chart',
    chart: {
      xField,
      series: finalSeries,
    },
  }
}

export function isNumericFieldType(fieldType: string | undefined): boolean {
  if (!fieldType) return false
  const u = fieldType.toUpperCase()
  return (
    u.includes('INT') ||
    u.includes('DECIMAL') ||
    u.includes('FLOAT') ||
    u.includes('DOUBLE') ||
    u.includes('NUM') ||
    u === 'REAL'
  )
}

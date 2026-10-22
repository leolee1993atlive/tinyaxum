import { appTableFeatures } from '#lib/table-features.js'
import { getInitialTableState } from '@tanstack/table-core'

/**
 * Full default state for the app's registered table features.
 *
 * v9 derives the state shape from the registered features, so this stays in
 * sync automatically as features are added or removed.
 */
export const TABLE_DEFAULT_STATE = getInitialTableState(appTableFeatures)

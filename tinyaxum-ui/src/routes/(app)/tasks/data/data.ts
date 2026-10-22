import { CircleOff, Loader } from '@lucide/svelte'

// Task statuses for transfer tasks
export const statuses = [
  {
    label: 'Stop',
    value: 'stop' as const,
    icon: CircleOff,
  },
  {
    label: 'Start',
    value: 'start' as const,
    icon: Loader,
  },
]

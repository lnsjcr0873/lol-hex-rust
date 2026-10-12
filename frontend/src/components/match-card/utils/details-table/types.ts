import type { VNodeChild } from 'vue'

export type RenderGroupOptions = {
  key: string
  hide?: boolean
  render?:
    | 'float'
    | 'akari-score'
    | 'integer'
    | 'text'
    | 'compat'
    | 'boolean'
    | 'game-time'
    | 'percentage'
    | 'position'
    | 'auto'
    | ((value: any) => VNodeChild)
}

export type RenderGroup = {
  group: string
  name?: string
  items: RenderGroupOptions[]
}

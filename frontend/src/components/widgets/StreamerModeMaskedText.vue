<template>
  <template v-if="shouldMask">
    <slot name="masked">
      <span>{{ fallbackMaskedText }}</span>
    </slot>
  </template>
  <template v-else>
    <slot>
      <span>{{ text }}</span>
    </slot>
  </template>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useStreamerModeMaskedText } from './useStreamerModeMaskedText'

const props = withDefaults(
  defineProps<{
    text?: string
    puuid?: string
    index?: number
    replacer?: string
    forceMask?: boolean
  }>(),
  {
    text: '',
    index: 0,
    forceMask: undefined
  }
)

const { isStreamerMode, summonerName, masked } = useStreamerModeMaskedText()

const shouldMask = computed(() => {
  if (props.forceMask !== undefined) return props.forceMask
  return isStreamerMode.value
})

const fallbackMaskedText = computed(() => {
  if (props.replacer) {
    return masked(props.text, props.replacer)
  }
  const seed = props.puuid || props.text || `p-${props.index}`
  return summonerName(seed, props.index)
})
</script>

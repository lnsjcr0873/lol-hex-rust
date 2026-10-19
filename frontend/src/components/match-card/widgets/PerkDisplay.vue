<template>
  <div
    class="relative inline-flex items-center justify-center shrink-0 overflow-hidden bg-black/60 rounded-full border border-[#1b263b]"
    :style="{ width: `${size}px`, height: `${size}px` }"
    :title="getPerkName(perkId)"
  >
    <img
      v-if="perkId && perkId > 0"
      :src="imgSrc"
      class="w-full h-full object-cover scale-110"
      :alt="getPerkName(perkId)"
      @error="handleImgError"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { getPerkIconUrl, getPerkName } from '../utils/resources'

const props = withDefaults(
  defineProps<{
    perkId?: number
    size?: number
  }>(),
  {
    perkId: 0,
    size: 20
  }
)

const fallbackIcon =
  'https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/v1/perk-images/styles/runesicon.png'

const imgSrc = ref(getPerkIconUrl(props.perkId))

watch(
  () => props.perkId,
  (newId) => {
    imgSrc.value = getPerkIconUrl(newId)
  }
)

function handleImgError() {
  if (imgSrc.value !== fallbackIcon) {
    imgSrc.value = fallbackIcon
  }
}
</script>


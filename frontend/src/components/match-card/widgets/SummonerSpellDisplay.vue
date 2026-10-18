<template>
  <div
    class="relative inline-flex items-center justify-center shrink-0 overflow-hidden bg-black/60 rounded-md border border-[#1b263b]"
    :style="{ width: `${size}px`, height: `${size}px` }"
    :title="getSpellName(spellId)"
  >
    <img
      v-if="spellId && spellId > 0"
      :src="imgSrc"
      class="w-full h-full object-cover"
      :alt="getSpellName(spellId)"
      @error="handleImgError"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { getSpellIconUrl, getSpellName } from '../utils/resources'

const props = withDefaults(
  defineProps<{
    spellId?: number
    size?: number
  }>(),
  {
    spellId: 0,
    size: 20
  }
)

const imgSrc = ref(getSpellIconUrl(props.spellId))

watch(
  () => props.spellId,
  (newId) => {
    imgSrc.value = getSpellIconUrl(newId)
  }
)

function handleImgError() {
  imgSrc.value = 'https://game.gtimg.cn/images/lol/act/img/spell/SummonerFlash.png'
}
</script>

<template>
  <div
    class="flex gap-3 rounded border border-indigo-500/20 bg-indigo-500/5 p-2.5 dark:border-indigo-300/15 dark:bg-indigo-950/20 text-slate-200"
  >
    <div class="flex shrink-0 flex-col gap-1.5">
      <slot name="map" />
    </div>

    <div class="flex min-w-0 flex-1 flex-col gap-2 text-xs">
      <slot name="content" />
    </div>
  </div>
</template>

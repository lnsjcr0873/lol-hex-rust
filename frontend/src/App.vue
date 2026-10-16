<template>
  <n-config-provider :theme="darkTheme">
    <n-message-provider>
      <n-dialog-provider>
        <OngoingOverlayView v-if="isOverlayWindow" />
        <div v-else class="flex flex-col h-screen w-screen overflow-hidden bg-[#090d1a] border border-[#162238] rounded-lg">
          <!-- Custom Draggable Titlebar -->
          <Titlebar :gameflowPhaseText="currentPhaseText" />

          <!-- Main Workspace: Left Sidebar + Center Viewport -->
          <div class="flex flex-1 overflow-hidden">
            <Sidebar 
              :activeTab="currentTab" 
              @change-tab="handleTabChange" 
              @refresh="handleRefresh"
            />

            <main class="flex-1 flex flex-col overflow-hidden relative">
              <OngoingMatch v-show="currentTab === 'ongoing'" ref="ongoingRef" @navigate-to-summoner="handleNavigateToSummoner" />
              <MatchHistory v-show="currentTab === 'history'" ref="historyRef" />
              <Automation v-show="currentTab === 'automation'" />
              <Settings v-show="currentTab === 'settings'" />
            </main>
          </div>
        </div>
      </n-dialog-provider>
    </n-message-provider>
  </n-config-provider>
</template>

<script setup lang="ts">
import { ref, nextTick, onMounted, onUnmounted } from 'vue';
import { NConfigProvider, NMessageProvider, NDialogProvider, darkTheme } from 'naive-ui';
import Titlebar from './components/Titlebar.vue';
import Sidebar from './components/Sidebar.vue';
import OngoingMatch from './views/OngoingMatch.vue';
import OngoingOverlayView from './views/OngoingOverlayView.vue';
import MatchHistory from './views/MatchHistory.vue';
import Automation from './views/Automation.vue';
import Settings from './views/Settings.vue';

const checkOverlay = () => {
  if (typeof window !== 'undefined') {
    return (
      window.location.search.includes('window=overlay') ||
      window.location.hash.includes('overlay')
    );
  }
  return false;
};

const isOverlayWindow = ref(checkOverlay());

const updateOverlayState = () => {
  isOverlayWindow.value = checkOverlay();
};

onMounted(() => {
  if (typeof window !== 'undefined') {
    window.addEventListener('hashchange', updateOverlayState);
    window.addEventListener('popstate', updateOverlayState);
  }
});

onUnmounted(() => {
  if (typeof window !== 'undefined') {
    window.removeEventListener('hashchange', updateOverlayState);
    window.removeEventListener('popstate', updateOverlayState);
  }
});

type Tab = 'ongoing' | 'history' | 'automation' | 'settings';

const currentTab = ref<Tab>('ongoing');
const currentPhaseText = ref('对局准备就绪');

const ongoingRef = ref<InstanceType<typeof OngoingMatch> | null>(null);
const historyRef = ref<InstanceType<typeof MatchHistory> | null>(null);

const handleTabChange = (tab: Tab) => {
  currentTab.value = tab;
};

const handleNavigateToSummoner = async (puuid: string) => {
  currentTab.value = 'history';
  await nextTick();
  if (historyRef.value && typeof (historyRef.value as any).loadSummoner === 'function') {
    (historyRef.value as any).loadSummoner(puuid);
  }
};

const handleRefresh = () => {
  if (currentTab.value === 'ongoing' && ongoingRef.value) {
    // Refresh ongoing
  } else if (currentTab.value === 'history' && historyRef.value) {
    // Refresh history
  }
};
</script>

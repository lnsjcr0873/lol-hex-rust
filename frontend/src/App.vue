<template>
  <n-config-provider :theme="darkTheme">
    <n-message-provider>
      <n-dialog-provider>
        <div class="flex flex-col h-screen w-screen overflow-hidden bg-[#090d1a] border border-[#162238] rounded-lg">
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
              <OngoingMatch v-show="currentTab === 'ongoing'" ref="ongoingRef" />
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
import { ref } from 'vue';
import { NConfigProvider, NMessageProvider, NDialogProvider, darkTheme } from 'naive-ui';
import Titlebar from './components/Titlebar.vue';
import Sidebar from './components/Sidebar.vue';
import OngoingMatch from './views/OngoingMatch.vue';
import MatchHistory from './views/MatchHistory.vue';
import Automation from './views/Automation.vue';
import Settings from './views/Settings.vue';

type Tab = 'ongoing' | 'history' | 'automation' | 'settings';

const currentTab = ref<Tab>('ongoing');
const currentPhaseText = ref('对局准备就绪');

const ongoingRef = ref<InstanceType<typeof OngoingMatch> | null>(null);
const historyRef = ref<InstanceType<typeof MatchHistory> | null>(null);

const handleTabChange = (tab: Tab) => {
  currentTab.value = tab;
};

const handleRefresh = () => {
  if (currentTab.value === 'ongoing' && ongoingRef.value) {
    // Refresh ongoing
  } else if (currentTab.value === 'history' && historyRef.value) {
    // Refresh history
  }
};
</script>

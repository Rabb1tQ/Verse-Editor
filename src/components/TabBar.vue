<template>
  <div class="tab-bar" v-if="tabs.length > 0">
    <div class="tab-list">
      <div
        v-for="(tab, index) in tabs"
        :key="tab.id"
        class="tab-item"
        :class="{ active: tab.id === activeTabId }"
        @click="selectTab(tab.id)"
        @contextmenu.prevent="showContextMenu($event, tab)"
      >
        <div class="tab-icon">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
            <path d="M14,2H6A2,2 0 0,0 4,4V20A2,2 0 0,0 6,22H18A2,2 0 0,0 20,20V8L14,2M18,20H6V4H13V9H18V20Z" />
          </svg>
        </div>
        
        <span class="tab-title" :title="tab.path || '未命名文档'">
          {{ tab.title }}
          <span v-if="tab.deleted" class="deleted-indicator">•</span>
          <span v-else-if="tab.modified" class="modified-indicator">•</span>
        </span>
        
        <button 
          class="tab-close"
          @click.stop="closeTab(tab.id)"
          :title="`关闭 ${tab.title}`"
        >
          <svg width="12" height="12" viewBox="0 0 24 24" fill="currentColor">
            <path d="M19,6.41L17.59,5L12,10.59L6.41,5L5,6.41L10.59,12L5,17.59L6.41,19L12,13.41L17.59,19L19,17.59L13.41,12L19,6.41Z" />
          </svg>
        </button>
      </div>
    </div>
    
    <div class="tab-actions">
      <button class="action-btn" @click="$emit('new-tab')" title="新建标签页">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
          <path d="M19,13H13V19H11V13H5V11H11V5H13V11H19V13Z" />
        </svg>
      </button>
    </div>
    
    <!-- 右键菜单 -->
    <div 
      v-if="contextMenu.visible"
      class="context-menu"
      :style="{ left: contextMenu.x + 'px', top: contextMenu.y + 'px' }"
      @click="hideContextMenu"
    >
      <div class="menu-item" @click="closeTab(contextMenu.tab.id)">
        关闭标签页
      </div>
      <div class="menu-item" @click="closeOtherTabs(contextMenu.tab.id)">
        关闭其他标签页
      </div>
      <div class="menu-item" @click="closeAllTabs">
        关闭所有标签页
      </div>
      <div class="menu-divider"></div>
      <div class="menu-item" @click="copyFilePath(contextMenu.tab.path)" :disabled="!contextMenu.tab.path">
        复制文件路径
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted } from 'vue'

const props = defineProps({
  tabs: {
    type: Array,
    default: () => []
  },
  activeTabId: {
    type: String,
    default: ''
  }
})

const emit = defineEmits([
  'tab-select',
  'tab-close',
  'tab-close-others',
  'tab-close-all',
  'new-tab'
])

const contextMenu = ref({
  visible: false,
  x: 0,
  y: 0,
  tab: null
})

// 选择标签页
const selectTab = (tabId) => {
  emit('tab-select', tabId)
}

// 关闭标签页
const closeTab = (tabId) => {
  emit('tab-close', tabId)
  hideContextMenu()
}

// 关闭其他标签页
const closeOtherTabs = (tabId) => {
  emit('tab-close-others', tabId)
  hideContextMenu()
}

// 关闭所有标签页
const closeAllTabs = () => {
  emit('tab-close-all')
  hideContextMenu()
}

// 显示右键菜单
const showContextMenu = (event, tab) => {
  contextMenu.value = {
    visible: true,
    x: event.clientX,
    y: event.clientY,
    tab
  }
}

// 隐藏右键菜单
const hideContextMenu = () => {
  contextMenu.value.visible = false
}

// 复制文件路径
const copyFilePath = async (path) => {
  if (path) {
    try {
      await navigator.clipboard.writeText(path)
    } catch (error) {
      console.error('复制文件路径失败:', error)
    }
  }
  hideContextMenu()
}

// 点击其他地方隐藏菜单
const handleGlobalClick = (event) => {
  if (!event.target.closest('.context-menu')) {
    hideContextMenu()
  }
}

onMounted(() => {
  document.addEventListener('click', handleGlobalClick)
})

onUnmounted(() => {
  document.removeEventListener('click', handleGlobalClick)
})
</script>

<style scoped>
.tab-bar {
  display: flex;
  align-items: center;
  background: var(--verse-bg-tertiary);
  border-bottom: 1px solid var(--verse-border-light);
  min-height: 36px;
  overflow: hidden;
}

.tab-list {
  flex: 1;
  display: flex;
  overflow-x: auto;
  scrollbar-width: none;
  -ms-overflow-style: none;
  gap: 0;
  padding: 0;
}

.tab-list::-webkit-scrollbar {
  display: none;
}

.tab-item {
  display: flex;
  align-items: center;
  padding: 8px 16px;
  background: var(--verse-bg-tertiary);
  border-right: 1px solid var(--verse-border-light);
  cursor: pointer;
  user-select: none;
  min-width: 120px;
  max-width: 200px;
  position: relative;
  transition: all 0.15s ease;
  font-size: 13px;
  font-weight: 400;
  color: var(--verse-text-primary);
  /* 移除标签页圆角，保持简洁 */
}

.tab-item:hover {
  background: var(--verse-bg-hover);
}

.tab-item.active {
  background: var(--verse-bg-secondary);
  color: var(--verse-text-primary);
  border-bottom: 2px solid var(--verse-accent);
  z-index: 1;
}

.tab-icon {
  display: flex;
  align-items: center;
  margin-right: var(--verse-spacing-sm);
  color: var(--verse-text-tertiary);
  flex-shrink: 0;
  transition: color 0.2s ease;
}

.tab-item.active .tab-icon {
  color: var(--verse-accent);
}

.tab-title {
  flex: 1;
  font-size: 13px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--verse-text-primary);
  font-weight: 500;
}

.tab-item.active .tab-title {
  color: var(--verse-accent);
  font-weight: 600;
}

.modified-indicator {
  color: var(--verse-warning);
  font-weight: bold;
  margin-left: var(--verse-spacing-xs);
  animation: pulse 2s infinite;
}

.deleted-indicator {
  color: #dc2626; /* 红色，表示文件已删除 */
  font-weight: bold;
  margin-left: var(--verse-spacing-xs);
  animation: pulse 2s infinite;
}

.tab-close {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 16px;
  height: 16px;
  border: none;
  background: transparent;
  border-radius: var(--verse-radius-xs);
  cursor: pointer;
  color: var(--verse-text-tertiary);
  margin-left: var(--verse-spacing-sm);
  flex-shrink: 0;
  opacity: 0;
  transition: all 0.2s ease;
}

.tab-item:hover .tab-close {
  opacity: 1;
}

.tab-close:hover {
  background: var(--verse-bg-tertiary);
  color: var(--verse-text-primary);
}

.tab-actions {
  display: flex;
  padding: 0 8px;
  border-left: 1px solid var(--verse-border-light);
  background: var(--verse-bg-tertiary);
}

.action-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border: none;
  background: transparent;
  border-radius: var(--verse-radius-sm);
  cursor: pointer;
  color: var(--verse-text-secondary);
  transition: all 0.15s ease;
}

.action-btn:hover {
  background: var(--verse-bg-hover);
  color: var(--verse-text-primary);
}

.context-menu {
  position: fixed;
  background: var(--verse-bg-secondary);
  border: 1px solid var(--verse-border-light);
  border-radius: var(--verse-radius-md);
  box-shadow: var(--verse-shadow-xl);
  z-index: var(--verse-z-dropdown);
  min-width: 160px;
  padding: var(--verse-spacing-xs) 0;
  backdrop-filter: blur(10px);
  -webkit-backdrop-filter: blur(10px);
}

.menu-item {
  padding: var(--verse-spacing-sm) var(--verse-spacing-md);
  cursor: pointer;
  font-size: 13px;
  color: var(--verse-text-primary);
  transition: all 0.2s ease;
  font-weight: 500;
}

.menu-item:hover:not([disabled]) {
  background: var(--verse-bg-hover);
}

.menu-item[disabled] {
  color: var(--verse-text-tertiary);
  cursor: not-allowed;
  opacity: 0.6;
}

.menu-divider {
  height: 1px;
  background: var(--verse-border-color);
  margin: var(--verse-spacing-xs) 0;
}

/* 主题样式由 CSS 变量统一管理 */
</style>

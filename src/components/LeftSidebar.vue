<template>
  <div 
    class="left-sidebar" 
    :class="{ collapsed: props.initialCollapsed }"
    :style="{ width: props.initialCollapsed ? '0' : `${sidebarWidth}px` }"
  >
    <div class="sidebar-header">
      <el-tabs v-model="activeTab" type="border-card" @tab-click="handleTabClick">
        <el-tab-pane label="文件" name="files">
          <template #label>
            <el-icon><Folder /></el-icon>
            <span class="tab-label">文件</span>
          </template>
        </el-tab-pane>
        <el-tab-pane label="大纲" name="outline">
          <template #label>
            <el-icon><List /></el-icon>
            <span class="tab-label">大纲</span>
          </template>
        </el-tab-pane>
      </el-tabs>
    </div>
    
    <div class="sidebar-content">
      <!-- 文件资源管理器 -->
      <div v-show="activeTab === 'files'" class="panel-content">
        <FileExplorer
          ref="fileExplorer"
          :current-file="currentFile"
          @file-select="$emit('file-select', $event)"
          @folder-change="$emit('folder-change', $event)"
          @file-deleted="$emit('file-deleted', $event)"
          @file-renamed="$emit('file-renamed', $event)"
        />
      </div>
      
      <!-- 文档大纲 -->
      <div v-show="activeTab === 'outline'" class="panel-content">
        <OutlineSidebar
          :content="content"
          @heading-click="$emit('heading-click', $event)"
        />
      </div>
    </div>
    
    <!-- 拖拽调整宽度的把手 -->
    <div 
      class="resize-handle"
      @mousedown="startResize"
      v-if="!props.initialCollapsed"
    ></div>
    
    <!-- 折叠按钮 -->
    <div class="collapse-toggle" @click="toggleCollapse">
      <el-icon>
        <component :is="props.initialCollapsed ? 'ArrowRight' : 'ArrowLeft'" />
      </el-icon>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted } from 'vue'
import { Folder, List, ArrowLeft, ArrowRight } from '@element-plus/icons-vue'
import FileExplorer from './FileExplorer.vue'
import OutlineSidebar from './OutlineSidebar.vue'

const props = defineProps({
  currentFile: {
    type: String,
    default: ''
  },
  content: {
    type: String,
    default: ''
  },
  initialCollapsed: {
    type: Boolean,
    default: false
  },
  initialWidth: {
    type: Number,
    default: 320
  }
})

const emit = defineEmits(['file-select', 'folder-change', 'heading-click', 'collapse-change', 'width-change'])

const activeTab = ref('files')
const sidebarWidth = ref(props.initialWidth)
const isResizing = ref(false)
const fileExplorer = ref(null)

// 拖拽调整宽度
const startResize = (e) => {
  isResizing.value = true
  document.addEventListener('mousemove', handleResize)
  document.addEventListener('mouseup', stopResize)
  document.body.style.cursor = 'col-resize'
  document.body.style.userSelect = 'none'
  e.preventDefault()
}

const handleResize = (e) => {
  if (!isResizing.value) return
  
  const newWidth = e.clientX
  const minWidth = 200
  const maxWidth = 600
  
  if (newWidth >= minWidth && newWidth <= maxWidth) {
    sidebarWidth.value = newWidth
    emit('width-change', newWidth)
  }
}

const stopResize = () => {
  isResizing.value = false
  document.removeEventListener('mousemove', handleResize)
  document.removeEventListener('mouseup', stopResize)
  document.body.style.cursor = ''
  document.body.style.userSelect = ''
}

const handleTabClick = (tab) => {
  console.log('Tab clicked:', tab.name)
}

const toggleCollapse = () => {
  emit('collapse-change', !props.initialCollapsed)
}

onMounted(() => {
  // 监听初始宽度变化
  sidebarWidth.value = props.initialWidth
})

onUnmounted(() => {
  // 清理事件监听
  if (isResizing.value) {
    stopResize()
  }
})

// 暴露方法
defineExpose({
  toggleCollapse,
  setActiveTab: (tab) => {
    activeTab.value = tab
  },
  fileExplorer
})
</script>

<style scoped>
.left-sidebar {
  display: flex;
  flex-direction: column;
  background: var(--verse-bg-secondary);
  border-right: 1px solid var(--verse-border-light);
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  position: relative;
  box-shadow: inset -1px 0 0 var(--verse-border-light);
  /* 移除圆角，保持现代桌面应用的直角设计 */
  overflow: hidden;
}

.left-sidebar:not(.collapsed) {
  min-width: 200px;
  max-width: 600px;
}

.left-sidebar.collapsed {
  width: 0 !important;
  min-width: 0 !important;
  max-width: 0 !important;
  overflow: hidden;
  border-right: none;
  box-shadow: none;
}

.sidebar-header {
  border-bottom: 1px solid var(--verse-border-light);
  flex-shrink: 0;
  background: var(--verse-bg-tertiary);
}

.sidebar-header :deep(.el-tabs) {
  margin: 0;
}

.sidebar-header :deep(.el-tabs__header) {
  margin: 0;
  background: transparent;
}

.sidebar-header :deep(.el-tabs__nav-wrap) {
  padding: 0;
}

.sidebar-header :deep(.el-tabs__item) {
  padding: var(--verse-spacing-md) var(--verse-spacing-lg);
  font-size: 13px;
  font-weight: 500;
  color: var(--verse-text-secondary);
  border: none;
  background: transparent;
  transition: all 0.2s ease;
}

.sidebar-header :deep(.el-tabs__item:hover) {
  color: var(--verse-text-primary);
  background: var(--verse-bg-hover);
}

.sidebar-header :deep(.el-tabs__item.is-active) {
  color: var(--verse-accent);
  background: var(--verse-bg-secondary);
}

.sidebar-header :deep(.el-tabs__active-bar) {
  background: var(--verse-accent);
  height: 2px;
}

.tab-label {
  margin-left: var(--verse-spacing-sm);
  font-weight: 500;
}

.sidebar-content {
  flex: 1;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  background: var(--verse-bg-secondary);
}

.panel-content {
  flex: 1;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.collapse-toggle {
  position: absolute;
  top: 50%;
  right: -12px;
  transform: translateY(-50%);
  width: 24px;
  height: 24px;
  background: var(--verse-bg-secondary);
  border: 1px solid var(--verse-border-color);
  border-radius: var(--verse-radius-sm);
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  z-index: 10;
  transition: all 0.2s ease;
  box-shadow: var(--verse-shadow-sm);
}

.collapse-toggle:hover {
  background: var(--verse-bg-hover);
  border-color: var(--verse-accent);
  box-shadow: var(--verse-shadow-md);
}

.collapse-toggle :deep(.el-icon) {
  font-size: 14px;
  color: var(--verse-text-secondary);
  transition: color 0.2s ease;
}

.collapse-toggle:hover :deep(.el-icon) {
  color: var(--verse-accent);
}

/* 添加一些微妙的动画效果 */
@keyframes slideIn {
  from {
    opacity: 0;
    transform: translateX(-20px);
  }
  to {
    opacity: 1;
    transform: translateX(0);
  }
}

.panel-content {
  animation: slideIn 0.3s ease-out;
}

/* 拖拽调整宽度的把手 - 简化设计 */
.resize-handle {
  position: absolute;
  top: 0;
  right: -2px;
  width: 4px;
  height: 100%;
  cursor: col-resize;
  background: transparent;
  z-index: 10;
  transition: all 0.2s ease;
}

.resize-handle:hover {
  background: var(--verse-accent);
}

.resize-handle::after {
  content: '';
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  width: 1px;
  height: 40px;
  background: var(--verse-border-color);
  opacity: 0;
  transition: all 0.2s ease;
}

.resize-handle:hover::after {
  opacity: 0.6;
  background: white;
  width: 2px;
  height: 50px;
}
</style>

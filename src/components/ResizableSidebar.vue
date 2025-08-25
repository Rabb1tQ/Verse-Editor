<template>
  <div 
    class="resizable-sidebar"
    :class="{ 
      'sidebar-left': position === 'left',
      'sidebar-right': position === 'right',
      'sidebar-collapsed': collapsed
    }"
    :style="{ 
      width: collapsed ? '0px' : width + 'px',
      minWidth: collapsed ? '0px' : minWidth + 'px',
      maxWidth: collapsed ? '0px' : maxWidth + 'px'
    }"
  >
    <!-- 侧边栏内容 -->
    <div class="sidebar-content" v-show="!collapsed">
      <div class="sidebar-tabs" v-if="tabs.length > 1">
        <button
          v-for="tab in tabs"
          :key="tab.id"
          class="tab-btn"
          :class="{ active: activeTab === tab.id }"
          @click="setActiveTab(tab.id)"
          :title="tab.title"
        >
          <component :is="tab.icon" width="16" height="16" />
          <span class="tab-label">{{ tab.label }}</span>
        </button>
      </div>
      
      <div class="sidebar-panel">
        <slot :active-tab="activeTab" />
      </div>
    </div>
    
    <!-- 调整大小手柄 -->
    <div 
      v-if="!collapsed"
      class="resize-handle"
      :class="{ 
        'handle-left': position === 'left',
        'handle-right': position === 'right'
      }"
      @mousedown="startResize"
    >
      <div class="handle-indicator"></div>
    </div>
    
    <!-- 折叠/展开按钮 -->
    <div class="collapse-btn-wrapper" :class="{ collapsed }">
      <button 
        class="collapse-btn"
        @click="toggleCollapse"
        :title="collapsed ? '展开侧边栏' : '折叠侧边栏'"
      >
        <svg 
          width="12" 
          height="12" 
          viewBox="0 0 24 24" 
          fill="currentColor"
          :style="{ 
            transform: getCollapseIconTransform() 
          }"
        >
          <path d="M8.59,16.58L13.17,12L8.59,7.41L10,6L16,12L10,18L8.59,16.58Z" />
        </svg>
      </button>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted } from 'vue'

const props = defineProps({
  position: {
    type: String,
    default: 'left', // 'left' | 'right'
    validator: (value) => ['left', 'right'].includes(value)
  },
  initialWidth: {
    type: Number,
    default: 280
  },
  minWidth: {
    type: Number,
    default: 200
  },
  maxWidth: {
    type: Number,
    default: 500
  },
  tabs: {
    type: Array,
    default: () => []
  },
  defaultTab: {
    type: String,
    default: ''
  },
  collapsible: {
    type: Boolean,
    default: true
  },
  initialCollapsed: {
    type: Boolean,
    default: false
  }
})

const emit = defineEmits(['width-change', 'collapse-change', 'tab-change'])

// 状态
const width = ref(props.initialWidth)
const collapsed = ref(props.initialCollapsed)
const activeTab = ref(props.defaultTab || (props.tabs.length > 0 ? props.tabs[0].id : ''))
const isResizing = ref(false)

// 调整大小相关
let startX = 0
let startWidth = 0

onMounted(() => {
  document.addEventListener('mousemove', handleResize)
  document.addEventListener('mouseup', stopResize)
})

onUnmounted(() => {
  document.removeEventListener('mousemove', handleResize)
  document.removeEventListener('mouseup', stopResize)
})

// 开始调整大小
const startResize = (event) => {
  isResizing.value = true
  startX = event.clientX
  startWidth = width.value
  document.body.style.cursor = 'col-resize'
  document.body.style.userSelect = 'none'
}

// 处理调整大小
const handleResize = (event) => {
  if (!isResizing.value) return
  
  const deltaX = event.clientX - startX
  let newWidth
  
  if (props.position === 'left') {
    newWidth = startWidth + deltaX
  } else {
    newWidth = startWidth - deltaX
  }
  
  // 限制宽度范围
  newWidth = Math.max(props.minWidth, Math.min(props.maxWidth, newWidth))
  
  width.value = newWidth
  emit('width-change', newWidth)
}

// 停止调整大小
const stopResize = () => {
  if (isResizing.value) {
    isResizing.value = false
    document.body.style.cursor = ''
    document.body.style.userSelect = ''
  }
}

// 切换折叠状态
const toggleCollapse = () => {
  collapsed.value = !collapsed.value
  emit('collapse-change', collapsed.value)
}

// 设置活动标签
const setActiveTab = (tabId) => {
  activeTab.value = tabId
  emit('tab-change', tabId)
}

// 获取折叠图标变换
const getCollapseIconTransform = () => {
  if (props.position === 'left') {
    return collapsed.value ? 'rotate(0deg)' : 'rotate(180deg)'
  } else {
    return collapsed.value ? 'rotate(180deg)' : 'rotate(0deg)'
  }
}

// 暴露方法
defineExpose({
  toggleCollapse,
  setWidth: (newWidth) => {
    width.value = Math.max(props.minWidth, Math.min(props.maxWidth, newWidth))
  },
  getWidth: () => width.value,
  isCollapsed: () => collapsed.value
})
</script>

<style scoped>
.resizable-sidebar {
  position: relative;
  display: flex;
  flex-direction: column;
  background: #f8f9fa;
  transition: width 0.3s ease;
  overflow: hidden;
}

.sidebar-left {
  border-right: 1px solid #e9ecef;
}

.sidebar-right {
  border-left: 1px solid #e9ecef;
}

.sidebar-collapsed {
  width: 0 !important;
  min-width: 0 !important;
  max-width: 0 !important;
}

.sidebar-content {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

.sidebar-tabs {
  display: flex;
  background: #ffffff;
  border-bottom: 1px solid #e9ecef;
  overflow-x: auto;
  scrollbar-width: none;
  -ms-overflow-style: none;
}

.sidebar-tabs::-webkit-scrollbar {
  display: none;
}

.tab-btn {
  display: flex;
  align-items: center;
  padding: 8px 12px;
  border: none;
  background: transparent;
  cursor: pointer;
  color: #6c757d;
  font-size: 12px;
  font-weight: 500;
  transition: all 0.2s ease;
  white-space: nowrap;
  border-bottom: 2px solid transparent;
}

.tab-btn:hover {
  background: #f8f9fa;
  color: #495057;
}

.tab-btn.active {
  color: #007bff;
  border-bottom-color: #007bff;
  background: #ffffff;
}

.tab-label {
  margin-left: 6px;
}

.sidebar-panel {
  flex: 1;
  overflow: hidden;
}

.resize-handle {
  position: absolute;
  top: 0;
  bottom: 0;
  width: 4px;
  cursor: col-resize;
  background: transparent;
  transition: background 0.2s ease;
  z-index: 10;
}

.handle-left {
  right: -2px;
}

.handle-right {
  left: -2px;
}

.resize-handle:hover {
  background: #007bff;
}

.resize-handle:hover .handle-indicator {
  opacity: 1;
}

.handle-indicator {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  width: 2px;
  height: 20px;
  background: #007bff;
  border-radius: 1px;
  opacity: 0;
  transition: opacity 0.2s ease;
}

.collapse-btn-wrapper {
  position: absolute;
  top: 12px;
  z-index: 20;
  transition: all 0.3s ease;
}

.sidebar-left .collapse-btn-wrapper {
  right: -12px;
}

.sidebar-right .collapse-btn-wrapper {
  left: -12px;
}

.collapse-btn-wrapper.collapsed {
  position: fixed;
}

.sidebar-left .collapse-btn-wrapper.collapsed {
  left: 8px;
}

.sidebar-right .collapse-btn-wrapper.collapsed {
  right: 8px;
}

.collapse-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  border: 1px solid #e9ecef;
  background: #ffffff;
  border-radius: 4px;
  cursor: pointer;
  color: #6c757d;
  transition: all 0.2s ease;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

.collapse-btn:hover {
  background: #f8f9fa;
  color: #495057;
  border-color: #007bff;
}

.collapse-btn svg {
  transition: transform 0.2s ease;
}

@media (prefers-color-scheme: dark) {
  .resizable-sidebar {
    background: #2d3748;
  }

  .sidebar-left {
    border-right-color: #4a5568;
  }

  .sidebar-right {
    border-left-color: #4a5568;
  }

  .sidebar-tabs {
    background: #2d3748;
    border-bottom-color: #4a5568;
  }

  .tab-btn {
    color: #a0aec0;
  }

  .tab-btn:hover {
    background: #4a5568;
    color: #f7fafc;
  }

  .tab-btn.active {
    color: #3182ce;
    border-bottom-color: #3182ce;
    background: #2d3748;
  }

  .resize-handle:hover {
    background: #3182ce;
  }

  .handle-indicator {
    background: #3182ce;
  }

  .collapse-btn {
    background: #2d3748;
    border-color: #4a5568;
    color: #a0aec0;
  }

  .collapse-btn:hover {
    background: #4a5568;
    color: #f7fafc;
    border-color: #3182ce;
  }
}
</style>

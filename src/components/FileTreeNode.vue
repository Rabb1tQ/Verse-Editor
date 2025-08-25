<template>
  <div class="file-tree-node">
    <div 
      class="node-content"
      :class="{ 
        'active': !node.isDirectory && node.path === currentFile,
        'directory': node.isDirectory
      }"
      @click="handleClick"
      @dblclick="handleDoubleClick"
      @contextmenu.prevent.stop="handleRightClick"
    >
      <div class="node-icon">
        <svg v-if="node.isDirectory" width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
          <path v-if="node.expanded" d="M19,20H4C2.89,20 2,19.1 2,18V6C2,4.89 2.89,4 4,4H10L12,6H19A2,2 0 0,1 21,8H21L4,8V18L6.2,12H23.2L21.4,18.8C21.2,19.5 20.5,20 19.7,20H19Z" />
          <path v-else d="M10,4H4C2.89,4 2,4.89 2,6V18A2,2 0 0,0 4,20H20A2,2 0 0,0 22,18V8C22,6.89 21.1,6 20,6H12L10,4Z" />
        </svg>
        <svg v-else width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
          <path d="M14,2H6A2,2 0 0,0 4,4V20A2,2 0 0,0 6,22H18A2,2 0 0,0 20,20V8L14,2M18,20H6V4H13V9H18V20Z" />
        </svg>
      </div>
      
      <div class="node-expand" v-if="node.isDirectory" @click.stop="toggleExpanded">
        <svg width="12" height="12" viewBox="0 0 24 24" fill="currentColor" 
             :style="{ transform: node.expanded ? 'rotate(90deg)' : 'rotate(0deg)' }">
          <path d="M8.59,16.58L13.17,12L8.59,7.41L10,6L16,12L10,18L8.59,16.58Z" />
        </svg>
      </div>
      
      <input 
        v-if="isRenaming" 
        ref="renameInput"
        v-model="renamingValue"
        class="node-name-input"
        @blur="confirmRename"
        @keydown.enter="confirmRename"
        @keydown.escape="cancelRename"
        @click.stop
      />
      <span v-else class="node-name" :title="node.name">{{ node.name }}</span>
    </div>
    
    <div v-if="node.isDirectory && node.expanded && node.children.length > 0" class="node-children">
      <FileTreeNode
        v-for="child in node.children"
        :key="child.path"
        :node="child"
        :current-file="currentFile"
        @file-click="$emit('file-click', $event)"
        @folder-toggle="$emit('folder-toggle', $event)"
        @context-menu="$emit('context-menu', $event)"
      />
    </div>
    
    <!-- 右键菜单 -->
    <div 
      v-if="showContextMenu" 
      class="context-menu"
      :style="{ top: contextMenuY + 'px', left: contextMenuX + 'px' }"
      @click.stop
    >
      <div class="menu-item" @click="handleCreateFile">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
          <path d="M14,2H6A2,2 0 0,0 4,4V20A2,2 0 0,0 6,22H18A2,2 0 0,0 20,20V8L14,2M18,20H6V4H13V9H18V20Z" />
        </svg>
        <span>新建文件</span>
      </div>
      <div class="menu-item" @click="handleCreateFolder">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
          <path d="M10,4H4C2.89,4 2,4.89 2,6V18A2,2 0 0,0 4,20H20A2,2 0 0,0 22,18V8C22,6.89 21.1,6 20,6H12L10,4Z" />
        </svg>
        <span>新建文件夹</span>
      </div>
      <div class="menu-divider"></div>
      <div v-if="!node.isDirectory" class="menu-item" @click="handleRename">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
          <path d="M20.71,7.04C21.1,6.65 21.1,6 20.71,5.63L18.37,3.29C18,2.9 17.35,2.9 16.96,3.29L15.12,5.12L18.87,8.87M3,17.25V21H6.75L17.81,9.93L14.06,6.18L3,17.25Z" />
        </svg>
        <span>重命名</span>
      </div>
      <div class="menu-item delete-item" @click="handleDelete">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
          <path d="M19,4H15.5L14.5,3H9.5L8.5,4H5V6H19M6,19A2,2 0 0,0 8,21H16A2,2 0 0,0 18,19V7H6V19Z" />
        </svg>
        <span>删除</span>
      </div>
      <div class="menu-item" @click="handleOpenLocation">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
          <path d="M10,4H4C2.89,4 2,4.89 2,6V18A2,2 0 0,0 4,20H20A2,2 0 0,0 22,18V8C22,6.89 21.1,6 20,6H12L10,4Z" />
        </svg>
        <span>打开文件位置</span>
      </div>
      <div class="menu-item" @click="handleCopyPath">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
          <path d="M19,21H8V7H19M19,5H8A2,2 0 0,0 6,7V21A2,2 0 0,0 8,23H19A2,2 0 0,0 21,21V7A2,2 0 0,0 19,5M16,1H4A2,2 0 0,0 2,3V17H4V3H16V1Z" />
        </svg>
        <span>复制路径</span>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted, nextTick } from 'vue'

const props = defineProps({
  node: {
    type: Object,
    required: true
  },
  currentFile: {
    type: String,
    default: ''
  }
})

const emit = defineEmits(['file-click', 'folder-toggle', 'context-menu'])

// 右键菜单状态
const showContextMenu = ref(false)
const contextMenuX = ref(0)
const contextMenuY = ref(0)

// 内联重命名状态
const isRenaming = ref(false)
const renamingValue = ref('')
const renameInput = ref(null)

const handleClick = () => {
  if (props.node.isDirectory) {
    toggleExpanded()
  } else {
    emit('file-click', props.node.path)
  }
}

const handleDoubleClick = () => {
  if (props.node.isDirectory) {
    // 双击文件夹时展开/折叠
    toggleExpanded()
  } else {
    emit('file-click', props.node.path)
  }
}

const toggleExpanded = () => {
  emit('folder-toggle', props.node)
}

// 右键菜单处理
const handleRightClick = (event) => {
  event.preventDefault()
  
  // 获取相对于视口的坐标
  const x = event.clientX
  const y = event.clientY
  
  // 确保菜单不会超出视口边界
  const menuWidth = 180 // 菜单估计宽度
  const menuHeight = 200 // 菜单估计高度
  
  contextMenuX.value = Math.min(x, window.innerWidth - menuWidth)
  contextMenuY.value = Math.min(y, window.innerHeight - menuHeight)
  showContextMenu.value = true
}

const hideContextMenu = () => {
  showContextMenu.value = false
}

// 菜单项处理函数
const handleCreateFile = () => {
  hideContextMenu()
  emit('context-menu', {
    action: 'create-file',
    node: props.node,
    targetPath: props.node.isDirectory ? props.node.path : getParentPath(props.node.path)
  })
}

const handleCreateFolder = () => {
  hideContextMenu()
  emit('context-menu', {
    action: 'create-folder',
    node: props.node,
    targetPath: props.node.isDirectory ? props.node.path : getParentPath(props.node.path)
  })
}

const handleRename = () => {
  hideContextMenu()
  startRename()
}

const handleOpenLocation = () => {
  hideContextMenu()
  emit('context-menu', {
    action: 'open-location',
    node: props.node
  })
}

const handleCopyPath = () => {
  hideContextMenu()
  emit('context-menu', {
    action: 'copy-path',
    node: props.node
  })
}

const handleDelete = () => {
  console.log('🎯 FileTreeNode handleDelete 被调用，文件:', props.node.name)
  hideContextMenu()
  console.log('📤 发出 context-menu 事件，action: delete')
  emit('context-menu', {
    action: 'delete',
    node: props.node
  })
}

// 获取父目录路径
const getParentPath = (filePath) => {
  return filePath.substring(0, filePath.lastIndexOf('/'))
}

// 点击其他地方隐藏菜单
const handleClickOutside = (event) => {
  if (showContextMenu.value) {
    hideContextMenu()
  }
}

onMounted(() => {
  document.addEventListener('click', handleClickOutside)
})

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside)
})

// 内联重命名功能
const startRename = () => {
  isRenaming.value = true
  renamingValue.value = props.node.name
  
  // 在下一个事件循环中聚焦输入框并选中文本
  nextTick(() => {
    if (renameInput.value) {
      renameInput.value.focus()
      // 选中文件名部分（不包括扩展名）
      const lastDotIndex = renamingValue.value.lastIndexOf('.')
      if (lastDotIndex > 0 && !props.node.isDirectory) {
        renameInput.value.setSelectionRange(0, lastDotIndex)
      } else {
        renameInput.value.select()
      }
    }
  })
}

const confirmRename = async () => {
  if (!isRenaming.value) return
  
  const newName = renamingValue.value.trim()
  if (!newName || newName === props.node.name) {
    cancelRename()
    return
  }
  
  try {
    emit('context-menu', {
      action: 'rename',
      node: props.node,
      newName: newName
    })
    isRenaming.value = false
  } catch (error) {
    console.error('重命名失败:', error)
    cancelRename()
  }
}

const cancelRename = () => {
  isRenaming.value = false
  renamingValue.value = ''
}
</script>

<style scoped>
.file-tree-node {
  user-select: none;
}

.node-content {
  display: flex;
  align-items: center;
  padding: 4px 8px 4px 16px;
  cursor: pointer;
  border-radius: 4px;
  margin: 1px 8px;
  transition: all 0.2s ease;
  position: relative;
}

.node-content:hover {
  background: rgba(0, 123, 255, 0.1);
}

.node-content.active {
  background: #007bff;
  color: white;
}

.node-content.directory {
  font-weight: 500;
}

.node-icon {
  display: flex;
  align-items: center;
  margin-right: 8px;
  color: #6c757d;
  flex-shrink: 0;
}

.node-content.active .node-icon {
  color: white;
}

.node-expand {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 16px;
  height: 16px;
  margin-right: 4px;
  border-radius: 2px;
  transition: all 0.2s ease;
  flex-shrink: 0;
}

.node-expand:hover {
  background: rgba(0, 0, 0, 0.1);
}

.node-expand svg {
  transition: transform 0.2s ease;
}

.node-name {
  flex: 1;
  font-size: 14px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.node-name-input {
  flex: 1;
  margin: 0 2px;
  padding: 2px 6px;
  font-size: 14px;
  color: var(--verse-text-primary, #374151);
  background: var(--verse-bg-primary, #ffffff);
  border: 1px solid var(--verse-primary, #3b82f6);
  border-radius: 4px;
  outline: none;
  font-family: inherit;
}

.node-name-input:focus {
  box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.2);
}

.node-children {
  padding-left: 12px;
  border-left: 1px solid #e9ecef;
  margin-left: 24px;
}

@media (prefers-color-scheme: dark) {
  .node-content:hover {
    background: rgba(59, 130, 246, 0.2);
  }

  .node-content.active {
    background: #3b82f6;
  }

  .node-icon {
    color: #a0aec0;
  }

  .node-expand:hover {
    background: rgba(255, 255, 255, 0.1);
  }

  .node-children {
    border-left-color: #4a5568;
  }
}

/* 右键菜单样式 */
.context-menu {
  position: fixed;
  background: var(--verse-bg-secondary, #ffffff);
  border: 1px solid var(--verse-border-light, #e5e7eb);
  border-radius: 8px;
  box-shadow: 0 10px 25px rgba(0, 0, 0, 0.15);
  padding: 6px 0;
  min-width: 180px;
  z-index: 1000;
  backdrop-filter: blur(10px);
  -webkit-backdrop-filter: blur(10px);
}

.menu-item {
  display: flex;
  align-items: center;
  padding: 8px 12px;
  cursor: pointer;
  color: var(--verse-text-primary, #374151);
  font-size: 14px;
  transition: all 0.15s ease;
  gap: 8px;
}

.menu-item:hover {
  background: var(--verse-bg-hover, #f3f4f6);
}

.menu-item svg {
  flex-shrink: 0;
  opacity: 0.7;
}

.menu-item span {
  flex: 1;
}

.menu-divider {
  height: 1px;
  background: var(--verse-border-light, #e5e7eb);
  margin: 4px 0;
}

.delete-item {
  color: #dc2626 !important;
}

.delete-item:hover {
  background: #fef2f2 !important;
  color: #dc2626 !important;
}

/* 暗色主题支持 */
@media (prefers-color-scheme: dark) {
  .context-menu {
    background: var(--verse-bg-secondary, #1f2937);
    border-color: var(--verse-border-light, #374151);
  }
  
  .menu-item {
    color: var(--verse-text-primary, #f9fafb);
  }
  
  .menu-item:hover {
    background: var(--verse-bg-hover, #374151);
  }
  
  .menu-divider {
    background: var(--verse-border-light, #374151);
  }
}
</style>

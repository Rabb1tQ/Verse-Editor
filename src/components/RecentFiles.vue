<template>
  <div class="recent-files">
    <div class="recent-header">
      <h3 class="recent-title">最近打开</h3>
      <div class="recent-actions">
        <button class="action-btn" @click="clearRecentFiles" title="清空列表" :disabled="recentFiles.length === 0">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <path d="M19,4H15.5L14.5,3H9.5L8.5,4H5V6H19M6,19A2,2 0 0,0 8,21H16A2,2 0 0,0 18,19V7H6V19Z" />
          </svg>
        </button>
        <button class="action-btn" @click="refreshRecentFiles" title="刷新">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <path d="M17.65,6.35C16.2,4.9 14.21,4 12,4A8,8 0 0,0 4,12A8,8 0 0,0 12,20C15.73,20 18.84,17.45 19.73,14H17.65C16.83,16.33 14.61,18 12,18A6,6 0 0,1 6,12A6,6 0 0,1 12,6C13.66,6 15.14,6.69 16.22,7.78L13,11H20V4L17.65,6.35Z" />
          </svg>
        </button>
      </div>
    </div>
    
    <div class="recent-content" v-if="recentFiles.length > 0">
      <div class="recent-list">
        <div
          v-for="(file, index) in recentFiles"
          :key="file.path"
          class="recent-item"
          :class="{ active: file.path === currentFile }"
          @click="openRecentFile(file)"
          @contextmenu.prevent="showContextMenu($event, file, index)"
        >
          <div class="file-icon">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
              <path d="M14,2H6A2,2 0 0,0 4,4V20A2,2 0 0,0 6,22H18A2,2 0 0,0 20,20V8L14,2M18,20H6V4H13V9H18V20Z" />
            </svg>
          </div>
          
          <div class="file-info">
            <div class="file-name" :title="file.name">{{ file.name }}</div>
            <div class="file-path" :title="file.path">{{ getDisplayPath(file.path) }}</div>
            <div class="file-time">{{ formatTime(file.lastOpened) }}</div>
          </div>
          
          <button 
            class="remove-btn"
            @click.stop="removeFromRecent(index)"
            title="从列表中移除"
          >
            <svg width="12" height="12" viewBox="0 0 24 24" fill="currentColor">
              <path d="M19,6.41L17.59,5L12,10.59L6.41,5L5,6.41L10.59,12L5,17.59L6.41,19L12,13.41L17.59,19L19,17.59L13.41,12L19,6.41Z" />
            </svg>
          </button>
        </div>
      </div>
    </div>
    
    <div class="recent-empty" v-else>
      <div class="empty-content">
        <svg width="48" height="48" viewBox="0 0 24 24" fill="currentColor" opacity="0.3">
          <path d="M12,2A10,10 0 0,0 2,12A10,10 0 0,0 12,22A10,10 0 0,0 22,12A10,10 0 0,0 12,2M12,4A8,8 0 0,1 20,12A8,8 0 0,1 12,20A8,8 0 0,1 4,12A8,8 0 0,1 12,4M12,6A6,6 0 0,0 6,12A6,6 0 0,0 12,18A6,6 0 0,0 18,12A6,6 0 0,0 12,6M12,8A4,4 0 0,1 16,12A4,4 0 0,1 12,16A4,4 0 0,1 8,12A4,4 0 0,1 12,8Z" />
        </svg>
        <p>暂无最近打开的文件</p>
      </div>
    </div>
    
    <!-- 右键菜单 -->
    <div 
      v-if="contextMenu.visible"
      class="context-menu"
      :style="{ left: contextMenu.x + 'px', top: contextMenu.y + 'px' }"
      @click="hideContextMenu"
    >
      <div class="menu-item" @click="openRecentFile(contextMenu.file)">
        打开文件
      </div>
      <div class="menu-item" @click="removeFromRecent(contextMenu.index)">
        从列表中移除
      </div>
      <div class="menu-divider"></div>
      <div class="menu-item" @click="copyFilePath(contextMenu.file.path)">
        复制文件路径
      </div>
      <div class="menu-item" @click="showInExplorer(contextMenu.file.path)">
        在文件管理器中显示
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted } from 'vue'

const props = defineProps({
  currentFile: {
    type: String,
    default: ''
  }
})

const emit = defineEmits(['file-select'])

// 最近文件列表
const recentFiles = ref([])
const maxRecentFiles = 20

// 右键菜单
const contextMenu = ref({
  visible: false,
  x: 0,
  y: 0,
  file: null,
  index: -1
})

onMounted(() => {
  loadRecentFiles()
  document.addEventListener('click', handleGlobalClick)
})

onUnmounted(() => {
  document.removeEventListener('click', handleGlobalClick)
})

// 加载最近文件
const loadRecentFiles = () => {
  try {
    const stored = localStorage.getItem('verse-recent-files')
    if (stored) {
      recentFiles.value = JSON.parse(stored)
    }
  } catch (error) {
    console.error('加载最近文件失败:', error)
    recentFiles.value = []
  }
}

// 保存最近文件
const saveRecentFiles = () => {
  try {
    localStorage.setItem('verse-recent-files', JSON.stringify(recentFiles.value))
  } catch (error) {
    console.error('保存最近文件失败:', error)
  }
}

// 添加文件到最近列表
const addToRecent = (filePath) => {
  if (!filePath) return
  
  const fileName = getFileName(filePath)
  const now = Date.now()
  
  // 移除已存在的相同文件
  const existingIndex = recentFiles.value.findIndex(file => file.path === filePath)
  if (existingIndex !== -1) {
    recentFiles.value.splice(existingIndex, 1)
  }
  
  // 添加到列表开头
  recentFiles.value.unshift({
    name: fileName,
    path: filePath,
    lastOpened: now
  })
  
  // 限制列表长度
  if (recentFiles.value.length > maxRecentFiles) {
    recentFiles.value = recentFiles.value.slice(0, maxRecentFiles)
  }
  
  saveRecentFiles()
}

// 从最近列表中移除
const removeFromRecent = (index) => {
  recentFiles.value.splice(index, 1)
  saveRecentFiles()
  hideContextMenu()
}

// 清空最近文件
const clearRecentFiles = () => {
  if (confirm('确定要清空最近打开的文件列表吗？')) {
    recentFiles.value = []
    saveRecentFiles()
  }
}

// 刷新最近文件
const refreshRecentFiles = () => {
  loadRecentFiles()
}

// 打开最近文件
const openRecentFile = (file) => {
  emit('file-select', file.path)
  addToRecent(file.path) // 更新时间
  hideContextMenu()
}

// 获取文件名
const getFileName = (filePath) => {
  const parts = filePath.split(/[\\/]/)
  return parts[parts.length - 1]
}

// 获取显示路径
const getDisplayPath = (filePath) => {
  const parts = filePath.split(/[\\/]/)
  if (parts.length > 3) {
    return '...' + parts.slice(-3, -1).join('/') + '/'
  }
  return parts.slice(0, -1).join('/') + '/'
}

// 格式化时间
const formatTime = (timestamp) => {
  const now = Date.now()
  const diff = now - timestamp
  
  const minute = 60 * 1000
  const hour = 60 * minute
  const day = 24 * hour
  const week = 7 * day
  
  if (diff < minute) {
    return '刚刚'
  } else if (diff < hour) {
    return `${Math.floor(diff / minute)} 分钟前`
  } else if (diff < day) {
    return `${Math.floor(diff / hour)} 小时前`
  } else if (diff < week) {
    return `${Math.floor(diff / day)} 天前`
  } else {
    return new Date(timestamp).toLocaleDateString()
  }
}

// 显示右键菜单
const showContextMenu = (event, file, index) => {
  contextMenu.value = {
    visible: true,
    x: event.clientX,
    y: event.clientY,
    file,
    index
  }
}

// 隐藏右键菜单
const hideContextMenu = () => {
  contextMenu.value.visible = false
}

// 复制文件路径
const copyFilePath = async (path) => {
  try {
    await navigator.clipboard.writeText(path)
  } catch (error) {
    console.error('复制文件路径失败:', error)
  }
  hideContextMenu()
}

// 在文件管理器中显示
const showInExplorer = (path) => {
  // 这里可以调用 Tauri API 来在系统文件管理器中显示文件
  console.log('在文件管理器中显示:', path)
  hideContextMenu()
}

// 处理全局点击
const handleGlobalClick = (event) => {
  if (!event.target.closest('.context-menu')) {
    hideContextMenu()
  }
}

// 暴露方法
defineExpose({
  addToRecent,
  clearRecentFiles
})
</script>

<style scoped>
.recent-files {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: #f8f9fa;
  border-right: 1px solid #e9ecef;
}

.recent-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-bottom: 1px solid #e9ecef;
  background: #ffffff;
}

.recent-title {
  font-size: 14px;
  font-weight: 600;
  color: #495057;
  margin: 0;
}

.recent-actions {
  display: flex;
  gap: 4px;
}

.action-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border: none;
  background: transparent;
  border-radius: 4px;
  cursor: pointer;
  color: #6c757d;
  transition: all 0.2s ease;
}

.action-btn:hover:not(:disabled) {
  background: #e9ecef;
  color: #495057;
}

.action-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.recent-content {
  flex: 1;
  overflow: hidden;
}

.recent-list {
  height: 100%;
  overflow-y: auto;
  padding: 8px 0;
}

.recent-item {
  display: flex;
  align-items: center;
  padding: 8px 16px;
  cursor: pointer;
  transition: background 0.2s ease;
  position: relative;
}

.recent-item:hover {
  background: rgba(0, 123, 255, 0.1);
}

.recent-item.active {
  background: #007bff;
  color: white;
}

.file-icon {
  display: flex;
  align-items: center;
  margin-right: 12px;
  color: #6c757d;
  flex-shrink: 0;
}

.recent-item.active .file-icon {
  color: white;
}

.file-info {
  flex: 1;
  min-width: 0;
}

.file-name {
  font-size: 13px;
  font-weight: 500;
  color: #495057;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  margin-bottom: 2px;
}

.recent-item.active .file-name {
  color: white;
}

.file-path {
  font-size: 11px;
  color: #6c757d;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  margin-bottom: 2px;
}

.recent-item.active .file-path {
  color: rgba(255, 255, 255, 0.8);
}

.file-time {
  font-size: 11px;
  color: #6c757d;
}

.recent-item.active .file-time {
  color: rgba(255, 255, 255, 0.8);
}

.remove-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
  border: none;
  background: transparent;
  border-radius: 2px;
  cursor: pointer;
  color: #6c757d;
  opacity: 0;
  transition: all 0.2s ease;
  flex-shrink: 0;
}

.recent-item:hover .remove-btn {
  opacity: 1;
}

.remove-btn:hover {
  background: rgba(0, 0, 0, 0.1);
  color: #495057;
}

.recent-item.active .remove-btn {
  color: rgba(255, 255, 255, 0.8);
}

.recent-item.active .remove-btn:hover {
  background: rgba(255, 255, 255, 0.2);
  color: white;
}

.recent-empty {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
}

.empty-content {
  text-align: center;
  color: #6c757d;
}

.empty-content svg {
  margin-bottom: 16px;
}

.empty-content p {
  margin: 0;
  font-size: 14px;
}

.context-menu {
  position: fixed;
  background: #ffffff;
  border: 1px solid #e9ecef;
  border-radius: 6px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  z-index: 1000;
  min-width: 160px;
  padding: 4px 0;
}

.menu-item {
  padding: 8px 16px;
  cursor: pointer;
  font-size: 14px;
  color: #495057;
  transition: background 0.2s ease;
}

.menu-item:hover {
  background: #f8f9fa;
}

.menu-divider {
  height: 1px;
  background: #e9ecef;
  margin: 4px 0;
}

@media (prefers-color-scheme: dark) {
  .recent-files {
    background: #2d3748;
    border-right-color: #4a5568;
  }

  .recent-header {
    background: #2d3748;
    border-bottom-color: #4a5568;
  }

  .recent-title {
    color: #e2e8f0;
  }

  .action-btn {
    color: #a0aec0;
  }

  .action-btn:hover:not(:disabled) {
    background: #4a5568;
    color: #f7fafc;
  }

  .recent-item:hover {
    background: rgba(59, 130, 246, 0.2);
  }

  .recent-item.active {
    background: #3b82f6;
  }

  .file-icon {
    color: #a0aec0;
  }

  .file-name {
    color: #e2e8f0;
  }

  .file-path,
  .file-time {
    color: #a0aec0;
  }

  .remove-btn {
    color: #a0aec0;
  }

  .remove-btn:hover {
    background: rgba(255, 255, 255, 0.1);
    color: #f7fafc;
  }

  .empty-content {
    color: #a0aec0;
  }

  .context-menu {
    background: #2d3748;
    border-color: #4a5568;
  }

  .menu-item {
    color: #e2e8f0;
  }

  .menu-item:hover {
    background: #4a5568;
  }

  .menu-divider {
    background: #4a5568;
  }
}
</style>

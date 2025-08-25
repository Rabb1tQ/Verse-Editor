<template>
  <div class="file-explorer">
    <div class="explorer-header">
      <h3 class="explorer-title">文件资源管理器</h3>
      <div class="explorer-actions">
        <button class="action-btn" @click="openFolder" title="打开文件夹">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <path d="M10,4H4C2.89,4 2,4.89 2,6V18A2,2 0 0,0 4,20H20A2,2 0 0,0 22,18V8C22,6.89 21.1,6 20,6H12L10,4Z" />
          </svg>
        </button>
        <button class="action-btn" @click="closeFolder" title="关闭文件夹" :disabled="!currentFolder">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <path d="M19,6.41L17.59,5L12,10.59L6.41,5L5,6.41L10.59,12L5,17.59L6.41,19L12,13.41L17.59,19L19,17.59L13.41,12L19,6.41Z" />
          </svg>
        </button>
        <button class="action-btn" @click="refreshFolder" title="刷新" :disabled="!currentFolder">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <path d="M17.65,6.35C16.2,4.9 14.21,4 12,4A8,8 0 0,0 4,12A8,8 0 0,0 12,20C15.73,20 18.84,17.45 19.73,14H17.65C16.83,16.33 14.61,18 12,18A6,6 0 0,1 6,12A6,6 0 0,1 12,6C13.66,6 15.14,6.69 16.22,7.78L13,11H20V4L17.65,6.35Z" />
          </svg>
        </button>
      </div>
    </div>
    
    <div class="explorer-content" v-if="currentFolder">
      <div class="folder-path">
        <span class="path-text" :title="currentFolder">{{ displayPath }}</span>
      </div>
      
      <div class="file-tree" @contextmenu.prevent="handleTreeRightClick">
        <FileTreeNode
          v-for="item in fileTree"
          :key="item.path"
          :node="item"
          :current-file="currentFile"
          @file-click="handleFileClick"
          @folder-toggle="handleFolderToggle"
          @context-menu="handleContextMenu"
        />
      </div>
    </div>
    
    <div class="explorer-empty" v-else>
      <div class="empty-content">
        <svg width="48" height="48" viewBox="0 0 24 24" fill="currentColor" opacity="0.3">
          <path d="M10,4H4C2.89,4 2,4.89 2,6V18A2,2 0 0,0 4,20H20A2,2 0 0,0 22,18V8C22,6.89 21.1,6 20,6H12L10,4Z" />
        </svg>
        <p>选择一个文件夹开始</p>
        <button class="open-folder-btn" @click="openFolder">打开文件夹</button>
      </div>
    </div>
    
    <!-- 空白区域右键菜单 -->
    <div 
      v-if="showTreeContextMenu" 
      class="context-menu"
      :style="{ top: treeContextMenuY + 'px', left: treeContextMenuX + 'px' }"
      @click.stop
    >
      <div class="menu-item" @click="handleTreeCreateFile">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
          <path d="M14,2H6A2,2 0 0,0 4,4V20A2,2 0 0,0 6,22H18A2,2 0 0,0 20,20V8L14,2M18,20H6V4H13V9H18V20Z" />
        </svg>
        <span>新建文件</span>
      </div>
      <div class="menu-item" @click="handleTreeCreateFolder">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
          <path d="M10,4H4C2.89,4 2,4.89 2,6V18A2,2 0 0,0 4,20H20A2,2 0 0,0 22,18V8C22,6.89 21.1,6 20,6H12L10,4Z" />
        </svg>
        <span>新建文件夹</span>
      </div>
      <div class="menu-divider"></div>
      <div class="menu-item" @click="handleTreeRefresh">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
          <path d="M17.65,6.35C16.2,4.9 14.21,4 12,4A8,8 0 0,0 4,12A8,8 0 0,0 12,20C15.73,20 18.84,17.45 19.73,14H17.65C16.83,16.33 14.61,18 12,18A6,6 0 0,1 6,12A6,6 0 0,1 12,6C13.66,6 15.14,6.69 16.22,7.78L13,11H20V4L17.65,6.35Z" />
        </svg>
        <span>刷新</span>
      </div>
      <div class="menu-item" @click="handleTreeOpenLocation">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
          <path d="M10,4H4C2.89,4 2,4.89 2,6V18A2,2 0 0,0 4,20H20A2,2 0 0,0 22,18V8C22,6.89 21.1,6 20,6H12L10,4Z" />
        </svg>
        <span>打开文件夹位置</span>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import { open, save, confirm } from '@tauri-apps/plugin-dialog'
import { readDir, writeTextFile, mkdir, rename, remove } from '@tauri-apps/plugin-fs'
import { openPath } from '@tauri-apps/plugin-opener'
import FileTreeNode from './FileTreeNode.vue'

const props = defineProps({
  currentFile: {
    type: String,
    default: ''
  }
})

const emit = defineEmits(['file-select', 'folder-change', 'file-deleted', 'file-renamed'])

const currentFolder = ref('')
const fileTree = ref([])

// 空白区域右键菜单状态
const showTreeContextMenu = ref(false)
const treeContextMenuX = ref(0)
const treeContextMenuY = ref(0)

// 保存目录展开状态
const expandedFolders = ref(new Set())

const displayPath = computed(() => {
  if (!currentFolder.value) return ''
  const parts = currentFolder.value.split(/[\\/]/)
  if (parts.length > 3) {
    return '...' + parts.slice(-2).join('/')
  }
  return parts.slice(-3).join('/')
})

// 打开文件夹
const openFolder = async () => {
  try {
    const selected = await open({
      directory: true,
      multiple: false
    })
    
    if (selected) {
      currentFolder.value = selected
      await loadFileTree()
      emit('folder-change', selected)
    }
  } catch (error) {
    console.error('打开文件夹失败:', error)
  }
}

// 关闭文件夹
const closeFolder = () => {
  currentFolder.value = ''
  fileTree.value = []
  emit('folder-change', '')
}

// 刷新文件夹
const refreshFolder = async () => {
  if (currentFolder.value) {
    await loadFileTree()
  }
}

const refreshFolderKeepExpanded = async () => {
  if (currentFolder.value) {
    await loadFileTree()
  }
}

// 加载文件树
const loadFileTree = async () => {
  if (!currentFolder.value) return
  
  try {
    const entries = await readDir(currentFolder.value, { recursive: false })
    fileTree.value = await processEntries(entries, currentFolder.value)
  } catch (error) {
    console.error('读取文件夹失败:', error)
  }
}

// 处理文件条目
const processEntries = async (entries, basePath) => {
  const processed = []
  
  for (const entry of entries) {
    // 确保路径正确处理 - 使用entry.path，如果没有则构造路径
    let entryPath = entry.path
    if (!entryPath) {
      // 标准化路径分隔符
      const normalizedBasePath = basePath.replace(/\\/g, '/')
      entryPath = `${normalizedBasePath}/${entry.name}`
    } else {
      // 标准化已有路径
      entryPath = entryPath.replace(/\\/g, '/')
    }
    
    const item = {
      name: entry.name,
      path: entryPath,
      isDirectory: entry.isDirectory,
      children: [],
      expanded: expandedFolders.value.has(entryPath) // 恢复展开状态
    }
    
    // 如果是文件夹且之前是展开状态，递归加载子项
    if (item.isDirectory && item.expanded) {
      try {
        const subEntries = await readDir(entryPath, { recursive: false })
        item.children = await processEntries(subEntries, entryPath)
      } catch (error) {
        console.error('读取子文件夹失败:', error)
        // 如果读取失败，设为收起状态并从展开记录中移除
        item.expanded = false
        expandedFolders.value.delete(entryPath)
      }
    }
    
    // 只显示 Markdown 文件和文件夹
    if (item.isDirectory || isMarkdownFile(item.name)) {
      processed.push(item)
    }
  }
  
  // 排序：文件夹在前，文件在后，按名称排序
  return processed.sort((a, b) => {
    if (a.isDirectory && !b.isDirectory) return -1
    if (!a.isDirectory && b.isDirectory) return 1
    return a.name.localeCompare(b.name)
  })
}

// 判断是否为 Markdown 文件
const isMarkdownFile = (filename) => {
  const ext = filename.toLowerCase().split('.').pop()
  return ['md', 'markdown', 'txt'].includes(ext)
}

// 处理文件点击
const handleFileClick = (filePath) => {
  emit('file-select', filePath)
}

// 处理文件夹展开/收起
const handleFolderToggle = async (node) => {
  if (node.isDirectory) {
    node.expanded = !node.expanded
    
    // 保存展开状态
    if (node.expanded) {
      expandedFolders.value.add(node.path)
    } else {
      expandedFolders.value.delete(node.path)
    }
    
    if (node.expanded && node.children.length === 0) {
      try {
        const entries = await readDir(node.path, { recursive: false })
        node.children = await processEntries(entries, node.path)
      } catch (error) {
        console.error('读取子文件夹失败:', error)
        console.error('文件夹路径:', node.path)
        // 避免过多弹窗，只在控制台记录错误
      }
    }
  }
}

// 监听当前文件夹变化
watch(currentFolder, (newFolder) => {
  if (newFolder) {
    loadFileTree()
  }
})

// 处理右键菜单事件
const handleContextMenu = async (menuData) => {
  const { action, node, targetPath } = menuData
  
  console.log('📥 FileExplorer handleContextMenu 接收到事件:', action, node?.name)
  
  try {
    switch (action) {
      case 'create-file':
        await handleCreateFile(targetPath)
        break
      case 'create-folder':
        await handleCreateFolder(targetPath)
        break
      case 'rename':
        await handleRename(node, menuData.newName)
        break
      case 'open-location':
        await handleOpenLocation(node)
        break
      case 'copy-path':
        await handleCopyPath(node)
        break
      case 'delete':
        handleDelete(node)
        break
    }
  } catch (error) {
    console.error('右键菜单操作失败:', error)
    alert(`操作失败: ${error.message}`)
  }
}

// 创建文件
const handleCreateFile = async (targetPath) => {
  const fileName = prompt('请输入文件名:', 'new-file.md')
  if (!fileName) return
  
  const filePath = `${targetPath}/${fileName}`.replace(/\\/g, '/')
  await writeTextFile(filePath, '')
  await refreshFolderKeepExpanded()
  emit('file-select', filePath)
}

// 创建文件夹
const handleCreateFolder = async (targetPath) => {
  const folderName = prompt('请输入文件夹名:', 'new-folder')
  if (!folderName) return
  
  const folderPath = `${targetPath}/${folderName}`.replace(/\\/g, '/')
  await mkdir(folderPath)
  await refreshFolderKeepExpanded()
}

// 重命名
const handleRename = async (node, newName) => {
  if (!newName || newName === node.name) return
  
  const parentPath = node.path.substring(0, node.path.lastIndexOf('/'))
  const newPath = `${parentPath}/${newName}`.replace(/\\/g, '/')
  
  await rename(node.path, newPath)
  
  // 通知App组件文件已重命名，用于标签页同步
  if (!node.isDirectory) {
    emit('file-renamed', { oldPath: node.path, newPath: newPath })
  }
  
  await refreshFolderKeepExpanded()
  
  // 如果是文件且当前打开，更新文件选择
  if (!node.isDirectory && node.path === props.currentFile) {
    emit('file-select', newPath)
  }
}

// 打开文件位置
const handleOpenLocation = async (node) => {
  try {
    if (node.isDirectory) {
      // 如果是文件夹，直接打开该文件夹
      await openPath(node.path)
    } else {
      // 如果是文件，打开文件所在目录并尝试选中文件
      const parentDir = node.path.substring(0, node.path.lastIndexOf('/'))
      
      // 在Windows上，可以使用explorer命令选中文件
      // 在其他系统上，只能打开目录
      const isWindows = navigator.platform.toLowerCase().includes('win')
      
      if (isWindows) {
        // Windows: 使用explorer /select 命令选中文件
        const windowsPath = node.path.replace(/\//g, '\\')
        await openPath(`explorer /select,"${windowsPath}"`)
      } else {
        // 其他系统：只打开目录
        await openPath(parentDir)
      }
    }
  } catch (error) {
    console.error('打开文件位置失败:', error)
    // 降级方案：直接打开目录
    const fallbackPath = node.isDirectory ? node.path : node.path.substring(0, node.path.lastIndexOf('/'))
    try {
      await openPath(fallbackPath)
    } catch (fallbackError) {
      console.error('降级方案也失败:', fallbackError)
      alert('无法打开文件位置')
    }
  }
}

// 复制路径
const handleCopyPath = async (node) => {
  try {
    await navigator.clipboard.writeText(node.path)
    console.log('路径已复制到剪贴板:', node.path)
  } catch (error) {
    console.error('复制路径失败:', error)
    // 降级方案：创建临时输入框
    const textArea = document.createElement('textarea')
    textArea.value = node.path
    document.body.appendChild(textArea)
    textArea.select()
    document.execCommand('copy')
    document.body.removeChild(textArea)
  }
}

// 删除文件/文件夹
const handleDelete = async (node) => {
  console.log('🚨 handleDelete 被调用，准备删除:', node.isDirectory ? '文件夹' : '文件', node.name)
  
  const itemType = node.isDirectory ? '文件夹' : '文件'
  const confirmMessage = `确定要删除${itemType} "${node.name}" 吗？\n\n此操作不可撤销！`
  
  console.log('🔔 即将显示确认对话框')
  
  // 使用Tauri的confirm对话框
  const userConfirmed = await confirm(confirmMessage, {
    title: '确认删除',
    type: 'warning'
  })
  console.log('✅ 用户确认结果:', userConfirmed)
  
  if (!userConfirmed) {
    console.log('❌ 用户取消删除操作，函数退出')
    return
  }
  
  console.log('🗑️ 用户确认删除，开始执行删除操作')
  
  try {
    // 如果删除的是文件夹，从展开状态中移除相关路径
    if (node.isDirectory) {
      // 移除该文件夹及其所有子路径的展开状态
      const pathsToRemove = []
      for (const expandedPath of expandedFolders.value) {
        if (expandedPath.startsWith(node.path)) {
          pathsToRemove.push(expandedPath)
        }
      }
      pathsToRemove.forEach(path => expandedFolders.value.delete(path))
    }
    
    await remove(node.path, { recursive: node.isDirectory })
    console.log(`${itemType}删除成功:`, node.path)
    
    // 通知App组件文件已删除，用于标签页同步
    if (!node.isDirectory) {
      emit('file-deleted', node.path)
    }
    
    await refreshFolderKeepExpanded()
    
    // 如果删除的是当前打开的文件，清除选择
    if (!node.isDirectory && node.path === props.currentFile) {
      emit('file-select', '')
    }
    
  } catch (error) {
    console.error(`删除${itemType}失败:`, error)
    alert(`删除${itemType}失败: ${error.message}`)
  }
}

// 空白区域右键菜单处理
const handleTreeRightClick = (event) => {
  event.preventDefault()
  
  // 获取相对于视口的坐标
  const rect = event.currentTarget.getBoundingClientRect()
  const x = event.clientX
  const y = event.clientY
  
  // 确保菜单不会超出视口边界
  const menuWidth = 180 // 菜单估计宽度
  const menuHeight = 160 // 菜单估计高度
  
  treeContextMenuX.value = Math.min(x, window.innerWidth - menuWidth)
  treeContextMenuY.value = Math.min(y, window.innerHeight - menuHeight)
  showTreeContextMenu.value = true
}

const hideTreeContextMenu = () => {
  showTreeContextMenu.value = false
}

// 空白区域菜单项处理
const handleTreeCreateFile = async () => {
  hideTreeContextMenu()
  await handleCreateFile(currentFolder.value)
}

const handleTreeCreateFolder = async () => {
  hideTreeContextMenu()
  await handleCreateFolder(currentFolder.value)
}

const handleTreeRefresh = async () => {
  hideTreeContextMenu()
  await refreshFolder()
}

const handleTreeOpenLocation = async () => {
  hideTreeContextMenu()
  await openPath(currentFolder.value)
}

// 点击外部隐藏菜单
const handleClickOutside = (event) => {
  if (showTreeContextMenu.value) {
    hideTreeContextMenu()
  }
}

// 添加全局点击监听
onMounted(() => {
  document.addEventListener('click', handleClickOutside)
})

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside)
})

// 暴露方法
defineExpose({
  openFolder,
  closeFolder,
  refreshFolder,
  currentFolder: computed(() => currentFolder.value)
})
</script>

<style scoped>
.file-explorer {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--verse-bg-secondary);
}

.explorer-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-bottom: 1px solid var(--verse-border-light);
  background: var(--verse-bg-tertiary);
}

.explorer-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--verse-text-primary);
  margin: 0;
}

.explorer-actions {
  display: flex;
  gap: 4px;
}

.action-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  border: none;
  background: transparent;
  border-radius: var(--verse-radius-sm);
  cursor: pointer;
  color: var(--verse-text-secondary);
  transition: all 0.15s ease;
}

.action-btn:hover:not(:disabled) {
  background: var(--verse-bg-hover);
  color: var(--verse-text-primary);
}

.action-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.explorer-content {
  flex: 1;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.folder-path {
  padding: 8px 16px;
  background: var(--verse-bg-secondary);
  border-bottom: 1px solid var(--verse-border-light);
}

.path-text {
  font-size: 12px;
  color: var(--verse-text-secondary);
  font-family: 'SF Mono', Monaco, 'Cascadia Code', monospace;
}

.file-tree {
  flex: 1;
  overflow-y: auto;
  padding: 8px 0;
}

.explorer-empty {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
}

.empty-content {
  text-align: center;
  color: var(--verse-text-secondary);
}

.empty-content svg {
  margin-bottom: 16px;
  color: var(--verse-text-tertiary);
}

.empty-content p {
  margin: 0 0 16px 0;
  font-size: 14px;
}

.open-folder-btn {
  padding: 8px 16px;
  border: 1px solid var(--verse-accent);
  background: var(--verse-accent);
  color: white;
  border-radius: var(--verse-radius-sm);
  cursor: pointer;
  font-size: 14px;
  transition: all 0.15s ease;
}

.open-folder-btn:hover {
  background: var(--verse-accent-hover);
  border-color: var(--verse-accent-hover);
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

<script setup>
import { ref, computed, onMounted, onUnmounted, nextTick } from 'vue'
import { open, save, ask, message } from '@tauri-apps/plugin-dialog'
import { readTextFile, writeTextFile } from '@tauri-apps/plugin-fs'
import { listen } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'
import MarkdownEditor from './components/MarkdownEditor.vue'
import MenuBar from './components/MenuBar.vue'
import LeftSidebar from './components/LeftSidebar.vue'
import TabBar from './components/TabBar.vue'
import SearchPanel from './components/SearchPanel.vue'

// 应用状态
const editorMode = ref('ir') // 'wysiwyg' | 'ir' | 'sv'
const currentFolder = ref('')

// 界面状态
const leftSidebarCollapsed = ref(false)
const leftSidebarWidth = ref(320)
const currentTheme = ref('light')
const recentFiles = ref([])

// 搜索面板状态
const searchPanelVisible = ref(false)

// 标签页管理
const tabs = ref([])
const activeTabId = ref('')
let tabIdCounter = 0

// 编辑器引用
const editorRef = ref(null)
const leftSidebarRef = ref(null)

// 计算属性
const activeTab = computed(() => {
  return tabs.value.find(tab => tab.id === activeTabId.value)
})

const currentFile = computed(() => activeTab.value?.path || '')
const content = computed(() => activeTab.value?.content || '')

// 监听快捷键
let unlistenKeydown = null
let unlistenFileOpen = null
let unlistenFileChanged = null
let unlistenWindowClose = null

onMounted(async () => {
  // 监听全局快捷键
  unlistenKeydown = await listen('tauri://menu', (event) => {
    handleMenuAction(event.payload)
  })

  // 监听文件打开事件（从命令行参数或双击文件）
  unlistenFileOpen = await listen('file-open', (event) => {
    const filePath = event.payload.path
    if (filePath) {
      console.log('收到文件打开事件:', filePath)
      handleFileSelect(filePath)
    }
  })

  // 监听文件变化事件（图片文件被创建、修改或删除）
  unlistenFileChanged = await listen('file-changed', (event) => {
    const { path, event_type } = event.payload
    console.log(`🔄 检测到图片文件变化: ${event_type} - ${path}`)
    
    // 刷新编辑器中的图片显示
    if (editorRef.value) {
      console.log('📸 正在刷新编辑器中的所有图片...')
      editorRef.value.refreshImages()
    } else {
      console.warn('⚠️ 编辑器引用不存在，无法刷新图片')
    }
  })

  // 监听键盘事件
  document.addEventListener('keydown', handleKeydown)
  
  // 监听窗口关闭事件（使用 Tauri API）
  const { getCurrentWindow } = await import('@tauri-apps/api/window')
  const appWindow = getCurrentWindow()
  
  unlistenWindowClose = await appWindow.onCloseRequested(async (event) => {
    // 检查是否有未保存的更改
    const unsavedTabs = tabs.value.filter(tab => tab.modified || tab.deleted)
    
    if (unsavedTabs.length > 0) {
      const names = unsavedTabs.map(tab => tab.title).join('、')
      const shouldClose = await ask(
        `以下文档有未保存的更改：${names}\n\n确定要关闭程序吗？`,
        {
          title: '确认退出',
          kind: 'warning'
        }
      )
      
      if (!shouldClose) {
        // 阻止窗口关闭
        event.preventDefault()
      }
    }
  })
  
  // 加载设置
  loadSettings()
  
  // 等待 Vue 完全准备好
  await nextTick()
  
  // 通知后端前端已准备好，可以发送启动文件事件
  try {
    const hasStartupFile = await invoke('frontend_ready')
    console.log('已通知后端前端准备就绪，是否有启动文件:', hasStartupFile)
    
    // 如果没有启动文件，创建欢迎页
    if (!hasStartupFile) {
      createNewTab('')
    }
  } catch (error) {
    console.error('通知后端失败:', error)
    // 如果通知失败，直接创建欢迎页
    createNewTab('')
  }
})

onUnmounted(() => {
  if (unlistenKeydown) {
    unlistenKeydown()
  }
  if (unlistenFileOpen) {
    unlistenFileOpen()
  }
  if (unlistenFileChanged) {
    unlistenFileChanged()
  }
  if (unlistenWindowClose) {
    unlistenWindowClose()
  }
  document.removeEventListener('keydown', handleKeydown)
})

// 标签页管理方法
const createNewTab = (initialContent = '', filePath = '') => {
  const tabId = `tab-${++tabIdCounter}`
  const fileName = filePath ? getFileName(filePath) : '未命名文档'
  
  const newTab = {
    id: tabId,
    title: fileName,
    path: filePath,
    content: initialContent,
    modified: false
  }
  
  tabs.value.push(newTab)
  activeTabId.value = tabId
  return newTab
}

const closeTab = async (tabId) => {
  const tabIndex = tabs.value.findIndex(tab => tab.id === tabId)
  if (tabIndex === -1) return
  
  const tab = tabs.value[tabIndex]
  
  // 检查是否有未保存的更改
  if (tab.modified) {
    // 使用 Tauri 的 ask 对话框（异步）
    const shouldClose = await ask(`文档 "${tab.title}" 有未保存的更改，是否关闭？`, {
      title: '确认关闭',
      kind: 'warning'
    })
    
    if (!shouldClose) {
      // 用户选择取消，不关闭标签页
      return
    }
  }
  
  // 移除标签页
  tabs.value.splice(tabIndex, 1)
  
  // 选择下一个标签页
  if (tabs.value.length > 0) {
    if (activeTabId.value === tabId) {
      const nextIndex = Math.min(tabIndex, tabs.value.length - 1)
      activeTabId.value = tabs.value[nextIndex].id
    }
  } else {
    activeTabId.value = ''
  }
}

const selectTab = (tabId) => {
  activeTabId.value = tabId
}

const getFileName = (filePath) => {
  if (!filePath) return '未命名文档'
  const parts = filePath.split(/[\\/]/)
  return parts[parts.length - 1]
}

// 处理键盘快捷键
const handleKeydown = (event) => {
  if (event.ctrlKey || event.metaKey) {
    switch (event.key) {
      case 'n':
        event.preventDefault()
        handleNewFile()
        break
      case 'o':
        event.preventDefault()
        handleOpenFile()
        break
      case 's':
        event.preventDefault()
        if (event.shiftKey) {
          handleSaveAs()
        } else {
          handleSaveFile()
        }
        break
      case 'w':
        event.preventDefault()
        if (activeTabId.value) {
          closeTab(activeTabId.value)
        }
        break
      case 'f':
        if (event.shiftKey) {
          event.preventDefault()
          toggleSearchPanel()
        }
        break
      case 'r':
        if (event.shiftKey) {
          // Ctrl+Shift+R 刷新图片
          event.preventDefault()
          if (editorRef.value) {
            console.log('🔄 手动刷新图片...')
            editorRef.value.refreshImages()
          }
        }
        break
    }
  }
}

// 处理菜单动作
const handleMenuAction = (action) => {
  switch (action) {
    case 'new':
      handleNewFile()
      break
    case 'open':
      handleOpenFile()
      break
    case 'save':
      handleSaveFile()
      break
    case 'save_as':
      handleSaveAs()
      break
  }
}

// 新建文件
const handleNewFile = () => {
  createNewTab()
}

// 打开文件
const handleOpenFile = async () => {
  try {
    const selected = await open({
      multiple: true,
      filters: [
        {
          name: 'Markdown',
          extensions: ['md', 'markdown', 'txt']
        }
      ]
    })

    if (selected) {
      const files = Array.isArray(selected) ? selected : [selected]
      
      for (const filePath of files) {
        // 检查文件是否已经打开
        const existingTab = tabs.value.find(tab => tab.path === filePath)
        if (existingTab) {
          activeTabId.value = existingTab.id
          continue
        }
        
        const fileContent = await readTextFile(filePath)
        createNewTab(fileContent, filePath)
        
        // 如果没有打开文件夹，自动监控文件所在的目录
        if (!currentFolder.value && filePath) {
          const fileDir = filePath.substring(0, Math.max(filePath.lastIndexOf('/'), filePath.lastIndexOf('\\')))
          if (fileDir) {
            try {
              console.log('🔍 自动监控文件所在目录:', fileDir)
              await invoke('watch_directory', { path: fileDir })
              console.log('✅ 目录文件监控已启动 (自动)', fileDir)
            } catch (error) {
              console.error('❌ 自动启动目录监控失败:', error)
            }
          }
        }
      }
    }
  } catch (error) {
    console.error('打开文件失败:', error)
    await message('打开文件失败: ' + error.message, { title: '错误', kind: 'error' })
  }
}

// 从文件资源管理器打开文件
const handleFileSelect = async (filePath, lineNumber = null) => {
  try {
    // 检查文件是否已经打开
    const existingTab = tabs.value.find(tab => tab.path === filePath)
    if (existingTab) {
      activeTabId.value = existingTab.id
      // TODO: 如果有行号，跳转到指定行
      if (lineNumber) {
        console.log('跳转到行号:', lineNumber)
      }
      return
    }
    
    const fileContent = await readTextFile(filePath)
    createNewTab(fileContent, filePath)
    
    // 添加到最近文件列表
    addToRecentFiles(filePath)
    
    // 如果没有打开文件夹，自动监控文件所在的目录
    if (!currentFolder.value && filePath) {
      const fileDir = filePath.substring(0, Math.max(filePath.lastIndexOf('/'), filePath.lastIndexOf('\\')))
      if (fileDir) {
        try {
          console.log('🔍 自动监控文件所在目录:', fileDir)
          await invoke('watch_directory', { path: fileDir })
          console.log('✅ 目录文件监控已启动 (自动)', fileDir)
        } catch (error) {
          console.error('❌ 自动启动目录监控失败:', error)
        }
      }
    }
    
    // TODO: 如果有行号，跳转到指定行
    if (lineNumber) {
      console.log('跳转到行号:', lineNumber)
    }
  } catch (error) {
    console.error('打开文件失败:', error)
    console.error('文件路径:', filePath)
    // 现在有了dialog权限，但避免过多弹窗，只在控制台显示错误
  }
}

// 保存文件
const handleSaveFile = async () => {
  const tab = activeTab.value
  if (!tab) return
  
  if (!tab.path) {
    return handleSaveAs()
  }

  try {
    const currentContent = editorRef.value?.getValue() || tab.content
    await writeTextFile(tab.path, currentContent)
    tab.modified = false
    tab.content = currentContent
  } catch (error) {
    console.error('保存文件失败:', error)
    await message('保存文件失败: ' + error.message, { title: '错误', kind: 'error' })
  }
}

// 另存为
const handleSaveAs = async () => {
  const tab = activeTab.value
  if (!tab) return
  
  try {
    const filePath = await save({
      filters: [
        {
          name: 'Markdown',
          extensions: ['md', 'markdown']
        },
        {
          name: 'Text',
          extensions: ['txt']
        }
      ]
    })

    if (filePath) {
      const currentContent = editorRef.value?.getValue() || tab.content
      await writeTextFile(filePath, currentContent)
      tab.path = filePath
      tab.title = getFileName(filePath)
      tab.modified = false
      tab.content = currentContent
    }
  } catch (error) {
    console.error('保存文件失败:', error)
    await message('保存文件失败: ' + error.message, { title: '错误', kind: 'error' })
  }
}

// 内容变化处理
const handleContentChange = (newContent) => {
  const tab = activeTab.value
  if (tab) {
    tab.content = newContent
    tab.modified = true
  }
}

// 保存快捷键处理
const handleSave = (value) => {
  const tab = activeTab.value
  if (tab) {
    tab.content = value
  }
  handleSaveFile()
}

// 切换编辑模式
const handleModeChange = (mode) => {
  editorMode.value = mode
}

// 标签页事件处理
const handleTabCloseOthers = async (keepTabId) => {
  const tabsToClose = tabs.value.filter(tab => tab.id !== keepTabId)
  const modifiedTabs = tabsToClose.filter(tab => tab.modified)
  
  if (modifiedTabs.length > 0) {
    const names = modifiedTabs.map(tab => tab.title).join('、')
    // 使用 Tauri 的 ask 对话框
    const shouldClose = await ask(`以下文档有未保存的更改：${names}，是否关闭这些标签页？`, {
      title: '确认关闭',
      kind: 'warning'
    })
    
    if (!shouldClose) {
      return
    }
  }
  
  tabs.value = tabs.value.filter(tab => tab.id === keepTabId)
  activeTabId.value = keepTabId
}

const handleTabCloseAll = async () => {
  const modifiedTabs = tabs.value.filter(tab => tab.modified)
  
  if (modifiedTabs.length > 0) {
    const names = modifiedTabs.map(tab => tab.title).join('、')
    // 使用 Tauri 的 ask 对话框
    const shouldClose = await ask(`以下文档有未保存的更改：${names}，是否关闭所有标签页？`, {
      title: '确认关闭',
      kind: 'warning'
    })
    
    if (!shouldClose) {
      return
    }
  }
  
  tabs.value = []
  activeTabId.value = ''
}

// 侧边栏切换
const toggleLeftSidebar = () => {
  leftSidebarVisible.value = !leftSidebarVisible.value
}

const toggleRightSidebar = () => {
  rightSidebarVisible.value = !rightSidebarVisible.value
}

// 文件夹变化处理
const handleFolderChange = async (folderPath) => {
  currentFolder.value = folderPath
  
  // 如果打开了文件夹，启动文件监控
  if (folderPath) {
    try {
      console.log('🔍 正在为目录启动文件监控:', folderPath)
      await invoke('watch_directory', { path: folderPath })
      console.log('✅ 目录文件监控已启动:', folderPath)
      console.log('💡 提示: 现在如果文件夹中的图片文件发生变化（创建/修改/删除），编辑器会自动刷新')
    } catch (error) {
      console.error('❌ 启动目录监控失败:', error)
    }
  }
}

// 处理文件删除
const handleFileDeleted = (filePath) => {
  // 找到对应的标签页
  const tabIndex = tabs.value.findIndex(tab => tab.path === filePath)
  if (tabIndex !== -1) {
    const tab = tabs.value[tabIndex]
    // 标记为已删除（未保存状态）
    tab.deleted = true
    tab.modified = true
    console.log('文件已删除，标签页标记为未保存:', filePath)
  }
}

// 处理文件重命名
const handleFileRenamed = ({ oldPath, newPath }) => {
  // 找到对应的标签页
  const tab = tabs.value.find(tab => tab.path === oldPath)
  if (tab) {
    // 更新标签页路径和名称
    tab.path = newPath
    tab.name = newPath.split('/').pop()
    tab.modified = true // 标记为需要保存到新位置
    console.log('文件已重命名，标签页已更新:', oldPath, '->', newPath)
    
    // 如果是当前活动标签页，更新选择
    if (activeTab.value?.path === oldPath) {
      activeTab.value.path = newPath
    }
  }
}

// 大纲点击处理
const handleHeadingClick = (item) => {
  if (editorRef.value) {
    // 通过行号滚动到对应位置
    editorRef.value.scrollToLine(item.line)
  }
}


// 检查是否有未保存的更改
const hasUnsavedChanges = () => {
  return tabs.value.some(tab => tab.modified || tab.deleted)
}

// 搜索面板相关
const toggleSearchPanel = () => {
  searchPanelVisible.value = !searchPanelVisible.value
  if (searchPanelVisible.value) {
    leftActiveTab.value = 'search'
  }
}

const closeSearchPanel = () => {
  searchPanelVisible.value = false
  if (leftActiveTab.value === 'search') {
    leftActiveTab.value = 'files'
  }
}

// 左侧边栏标签页配置
const leftSidebarTabs = computed(() => {
  const baseTabs = [
    {
      id: 'files',
      label: '文件',
      title: '文件资源管理器',
      icon: 'svg' // 这里应该是图标组件
    },
    {
      id: 'recent',
      label: '最近',
      title: '最近打开的文件',
      icon: 'svg'
    }
  ]
  
  if (searchPanelVisible.value) {
    baseTabs.push({
      id: 'search',
      label: '搜索',
      title: '全局搜索',
      icon: 'svg'
    })
  }
  
  return baseTabs
})

// 右侧边栏标签页配置
const rightSidebarTabs = computed(() => [
  {
    id: 'outline',
    label: '大纲',
    title: '文档大纲',
    icon: 'svg'
  }
])

// 侧边栏事件处理
const handleLeftSidebarCollapse = (collapsed) => {
  leftSidebarCollapsed.value = collapsed
}

const handleRightSidebarCollapse = (collapsed) => {
  rightSidebarCollapsed.value = collapsed
}

const handleLeftTabChange = (tabId) => {
  leftActiveTab.value = tabId
}

// 设置管理
const loadSettings = () => {
  try {
    const settings = localStorage.getItem('verse-settings')
    if (settings) {
      const parsed = JSON.parse(settings)
      currentTheme.value = parsed.theme || 'light'
      leftSidebarWidth.value = parsed.sidebarWidth || 320
      leftSidebarCollapsed.value = parsed.sidebarCollapsed || false
      applyTheme(currentTheme.value)
    }
    
    const recent = localStorage.getItem('verse-recent-files')
    if (recent) {
      recentFiles.value = JSON.parse(recent)
    }
  } catch (error) {
    console.error('加载设置失败:', error)
  }
}

const saveSettings = () => {
  try {
    const settings = {
      theme: currentTheme.value,
      sidebarWidth: leftSidebarWidth.value,
      sidebarCollapsed: leftSidebarCollapsed.value
    }
    localStorage.setItem('verse-settings', JSON.stringify(settings))
    localStorage.setItem('verse-recent-files', JSON.stringify(recentFiles.value))
  } catch (error) {
    console.error('保存设置失败:', error)
  }
}

// 主题管理
const applyTheme = (theme) => {
  const html = document.documentElement
  if (theme === 'dark') {
    html.classList.add('dark')
  } else if (theme === 'light') {
    html.classList.remove('dark')
  } else { // auto
    const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches
    if (prefersDark) {
      html.classList.add('dark')
    } else {
      html.classList.remove('dark')
    }
  }
}

const handleThemeChange = (theme) => {
  currentTheme.value = theme
  applyTheme(theme)
  saveSettings()
}

// 最近文件管理
const addToRecentFiles = (filePath) => {
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
  if (recentFiles.value.length > 20) {
    recentFiles.value = recentFiles.value.slice(0, 20)
  }
  
  saveSettings()
}

const handleOpenRecent = (filePath) => {
  handleFileSelect(filePath)
}

const handleClearRecent = () => {
  recentFiles.value = []
  saveSettings()
}

// 菜单栏事件处理
const handleOpenFolder = () => {
  // 触发左侧边栏的FileExplorer组件打开文件夹
  if (leftSidebarRef.value) {
    // 先确保侧边栏展开并切换到文件标签
    if (leftSidebarCollapsed.value) {
      leftSidebarCollapsed.value = false
      saveSettings()
    }
    leftSidebarRef.value.setActiveTab('files')
    
    // 调用FileExplorer的openFolder方法
    if (leftSidebarRef.value.fileExplorer) {
      leftSidebarRef.value.fileExplorer.openFolder()
    }
  }
}

const handleToggleSidebar = () => {
  leftSidebarCollapsed.value = !leftSidebarCollapsed.value
  saveSettings()
}

const handleToggleOutline = () => {
  if (leftSidebarCollapsed.value) {
    // 如果边栏是折叠的，先展开边栏然后切换到大纲标签
    leftSidebarCollapsed.value = false
    saveSettings()
  }
  // 切换到大纲标签
  leftSidebarRef.value?.setActiveTab('outline')
}

const handleGlobalSearch = () => {
  toggleSearchPanel()
}

// 侧边栏事件处理
const handleSidebarCollapse = (collapsed) => {
  leftSidebarCollapsed.value = collapsed
  saveSettings()
}

const handleSidebarWidthChange = (width) => {
  leftSidebarWidth.value = width
  saveSettings()
}
</script>

<template>
  <div class="app">
    <!-- 菜单栏 -->
    <MenuBar
      :mode="editorMode"
      :current-theme="currentTheme"
      :recent-files="recentFiles"
      @new-file="handleNewFile"
      @open-file="handleOpenFile"
      @open-folder="handleOpenFolder"
      @open-recent="handleOpenRecent"
      @clear-recent="handleClearRecent"
      @save-file="handleSaveFile"
      @save-as="handleSaveAs"
      @toggle-sidebar="handleToggleSidebar"
      @toggle-outline="handleToggleOutline"
      @change-mode="handleModeChange"
      @change-theme="handleThemeChange"
      @global-search="handleGlobalSearch"
    />
    
    <!-- 标签页栏 -->
    <TabBar
      :tabs="tabs"
      :active-tab-id="activeTabId"
      @tab-select="selectTab"
      @tab-close="closeTab"
      @tab-close-others="handleTabCloseOthers"
      @tab-close-all="handleTabCloseAll"
      @new-tab="handleNewFile"
    />
    
    <!-- 主内容区域 -->
    <div class="main-content">
      <!-- 左侧边栏 -->
      <LeftSidebar
        ref="leftSidebarRef"
        :current-file="currentFile"
        :content="activeTab?.content || ''"
        :initial-collapsed="leftSidebarCollapsed"
        :initial-width="leftSidebarWidth"
        @file-select="handleFileSelect"
        @folder-change="handleFolderChange"
        @heading-click="handleHeadingClick"
        @collapse-change="handleSidebarCollapse"
        @width-change="handleSidebarWidthChange"
      />
      
      <!-- 编辑器区域 -->
      <div class="editor-area" v-if="activeTab">
        <MarkdownEditor
          ref="editorRef"
          :content="activeTab.content"
          :mode="editorMode"
          :current-file-path="activeTab.path"
          @update:content="handleContentChange"
          @save="handleSave"
        />
      </div>
      
      <!-- 空状态 -->
      <div class="empty-editor" v-else>
        <div class="empty-content">
          <el-icon size="64" color="#dcdfe6">
            <Document />
          </el-icon>
          <div class="empty-actions">
            <el-button type="primary" @click="handleNewFile">新建文档</el-button>
            <el-button @click="handleOpenFile">打开文件</el-button>
          </div>
        </div>
      </div>
    </div>
    
    <!-- 搜索面板 -->
    <el-drawer
      v-model="searchPanelVisible"
      title="全局搜索"
      direction="ltr"
      size="400px"
    >
      <SearchPanel
        :visible="searchPanelVisible"
        :current-folder="currentFolder"
        @close="closeSearchPanel"
        @open-file="handleFileSelect"
      />
    </el-drawer>
  </div>
</template>

<style scoped>
.app {
  display: flex;
  flex-direction: column;
  height: 100vh;
  overflow: hidden;
}

.main-content {
  display: flex;
  flex: 1;
  overflow: hidden;
}



.editor-area {
  flex: 1;
  overflow: hidden;
  background: var(--verse-bg-secondary);
  /* 移除编辑器区域的圆角和边距，保持简洁 */
}

.empty-editor {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--verse-bg-secondary);
}

.empty-content {
  text-align: center;
  color: var(--verse-text-secondary);
  max-width: 400px;
}

.empty-content h3 {
  font-size: 24px;
  font-weight: 600;
  margin: 16px 0 12px 0;
  color: var(--verse-text-primary);
}

.empty-content p {
  font-size: 16px;
  margin: 0 0 32px 0;
  line-height: 1.5;
  color: var(--verse-text-secondary);
}

.empty-actions {
  display: flex;
  gap: 12px;
  justify-content: center;
}

/* 主题样式由 Element Plus 自动处理 */
</style>

<style>
/* Vditor 样式覆盖 */
.vditor {
  border: none !important;
  background: var(--verse-bg-primary) !important;
}

.vditor-toolbar {
  background: var(--verse-bg-secondary) !important;
  border-bottom: 1px solid var(--verse-border-color) !important;
  padding: var(--verse-spacing-sm) var(--verse-spacing-md) !important;
}

.vditor-toolbar .vditor-toolbar__item {
  color: var(--verse-text-secondary) !important;
  border-radius: var(--verse-radius-sm) !important;
  transition: all 0.2s ease !important;
}

.vditor-toolbar .vditor-toolbar__item:hover {
  background: var(--verse-bg-tertiary) !important;
  color: var(--verse-text-primary) !important;
}

.vditor-content {
  background: var(--verse-bg-primary) !important;
  color: var(--verse-text-primary) !important;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', 'Helvetica Neue', Arial, sans-serif !important;
  line-height: 1.6 !important;
}

.vditor-ir {
  background: var(--verse-bg-primary) !important;
}

.vditor-ir .vditor-ir__marker {
  color: var(--verse-text-tertiary) !important;
}

.vditor-wysiwyg {
  background: var(--verse-bg-primary) !important;
}

.vditor-sv {
  background: var(--verse-bg-primary) !important;
}

.vditor-preview {
  background: var(--verse-bg-primary) !important;
  color: var(--verse-text-primary) !important;
}

/* Element Plus 主题覆盖 */
.el-drawer {
  background: var(--verse-bg-primary) !important;
}

.el-drawer__header {
  background: var(--verse-bg-secondary) !important;
  border-bottom: 1px solid var(--verse-border-color) !important;
  padding: var(--verse-spacing-md) var(--verse-spacing-lg) !important;
}

.el-drawer__title {
  color: var(--verse-text-primary) !important;
  font-weight: 600 !important;
}

.el-button {
  border-radius: var(--verse-radius-md) !important;
  font-weight: 500 !important;
  transition: all 0.2s ease !important;
}

.el-button--primary {
  background: var(--verse-accent) !important;
  border-color: var(--verse-accent) !important;
}

.el-button--primary:hover {
  background: var(--verse-accent-hover) !important;
  border-color: var(--verse-accent-hover) !important;
}
</style>

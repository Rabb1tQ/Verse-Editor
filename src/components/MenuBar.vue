<template>
  <div class="menu-bar">
    <!-- 主菜单 -->
    <div class="menu-section">
      <div class="menu-item" @click="toggleMenu('file')" ref="fileMenuRef">
        <span class="menu-label">文件</span>
        <div v-if="activeMenu === 'file'" class="dropdown-menu">
          <div class="menu-option" @click="$emit('new-file')">
            <span>新建</span>
            <span class="shortcut">Ctrl+N</span>
          </div>
          <div class="menu-option" @click="$emit('open-file')">
            <span>打开文件</span>
            <span class="shortcut">Ctrl+O</span>
          </div>
          <div class="menu-option" @click="$emit('open-folder')">
            <span>打开文件夹</span>
            <span class="shortcut">Ctrl+Shift+O</span>
          </div>
          <div class="menu-divider"></div>
          <div class="menu-option recent-files" @click="toggleRecentFiles">
            <span>最近打开</span>
            <svg width="12" height="12" viewBox="0 0 24 24" fill="currentColor" class="arrow-icon">
              <path d="M8.59,16.58L13.17,12L8.59,7.41L10,6L16,12L10,18L8.59,16.58Z" />
            </svg>
          </div>
          <div v-if="showRecentFiles" class="recent-files-submenu">
            <div 
              v-for="file in recentFiles.slice(0, 10)" 
              :key="file.path"
              class="menu-option recent-item"
              @click="$emit('open-recent', file.path)"
            >
              <span class="file-name">{{ getFileName(file.name) }}</span>
              <span class="file-path">{{ getDisplayPath(file.path) }}</span>
            </div>
            <div v-if="recentFiles.length === 0" class="menu-option disabled">
              <span>暂无最近文件</span>
            </div>
            <div class="menu-divider"></div>
            <div class="menu-option" @click="$emit('clear-recent')">
              <span>清空列表</span>
            </div>
          </div>
          <div class="menu-divider"></div>
          <div class="menu-option" @click="$emit('save-file')">
            <span>保存</span>
            <span class="shortcut">Ctrl+S</span>
          </div>
          <div class="menu-option" @click="$emit('save-as')">
            <span>另存为</span>
            <span class="shortcut">Ctrl+Shift+S</span>
          </div>
        </div>
      </div>

      <div class="menu-item" @click="toggleMenu('edit')">
        <span class="menu-label">编辑</span>
        <div v-if="activeMenu === 'edit'" class="dropdown-menu">
          <div class="menu-option" @click="$emit('undo')">
            <span>撤销</span>
            <span class="shortcut">Ctrl+Z</span>
          </div>
          <div class="menu-option" @click="$emit('redo')">
            <span>重做</span>
            <span class="shortcut">Ctrl+Y</span>
          </div>
          <div class="menu-divider"></div>
          <div class="menu-option" @click="$emit('find')">
            <span>查找</span>
            <span class="shortcut">Ctrl+F</span>
          </div>
          <div class="menu-option" @click="$emit('find-replace')">
            <span>查找替换</span>
            <span class="shortcut">Ctrl+H</span>
          </div>
          <div class="menu-option" @click="$emit('global-search')">
            <span>全局搜索</span>
            <span class="shortcut">Ctrl+Shift+F</span>
          </div>
        </div>
      </div>

      <div class="menu-item" @click="toggleMenu('view')">
        <span class="menu-label">视图</span>
        <div v-if="activeMenu === 'view'" class="dropdown-menu">
          <div class="menu-option" @click="$emit('toggle-sidebar')">
            <span>切换侧边栏</span>
            <span class="shortcut">Ctrl+\\</span>
          </div>
          <div class="menu-option" @click="$emit('toggle-outline')">
            <span>切换大纲</span>
          </div>
          <div class="menu-divider"></div>
          <div class="menu-option mode-option" @click="$emit('change-mode', 'wysiwyg')" :class="{ active: mode === 'wysiwyg' }">
            <span>所见即所得</span>
          </div>
          <div class="menu-option mode-option" @click="$emit('change-mode', 'ir')" :class="{ active: mode === 'ir' }">
            <span>即时渲染</span>
          </div>
          <div class="menu-option mode-option" @click="$emit('change-mode', 'sv')" :class="{ active: mode === 'sv' }">
            <span>分屏预览</span>
          </div>
        </div>
      </div>

      <div class="menu-item" @click="toggleMenu('theme')">
        <span class="menu-label">主题</span>
        <div v-if="activeMenu === 'theme'" class="dropdown-menu">
          <div class="menu-option" @click="$emit('change-theme', 'light')" :class="{ active: currentTheme === 'light' }">
            <span>浅色主题</span>
          </div>
          <div class="menu-option" @click="$emit('change-theme', 'dark')" :class="{ active: currentTheme === 'dark' }">
            <span>深色主题</span>
          </div>
          <div class="menu-option" @click="$emit('change-theme', 'auto')" :class="{ active: currentTheme === 'auto' }">
            <span>跟随系统</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted } from 'vue'

const props = defineProps({
  mode: {
    type: String,
    default: 'ir'
  },
  currentTheme: {
    type: String,
    default: 'light'
  },
  recentFiles: {
    type: Array,
    default: () => []
  }
})

const emit = defineEmits([
  'new-file', 'open-file', 'open-folder', 'open-recent', 'clear-recent',
  'save-file', 'save-as', 'undo', 'redo', 'find', 'find-replace', 'global-search',
  'toggle-sidebar', 'toggle-outline', 'change-mode', 'change-theme'
])

const activeMenu = ref('')
const showRecentFiles = ref(false)
const fileMenuRef = ref(null)

const toggleMenu = (menuName) => {
  if (activeMenu.value === menuName) {
    activeMenu.value = ''
    showRecentFiles.value = false
  } else {
    activeMenu.value = menuName
    showRecentFiles.value = false
  }
}

const toggleRecentFiles = (event) => {
  event.stopPropagation()
  showRecentFiles.value = !showRecentFiles.value
}

const closeMenus = () => {
  activeMenu.value = ''
  showRecentFiles.value = false
}

const handleGlobalClick = (event) => {
  if (!event.target.closest('.menu-bar')) {
    closeMenus()
  }
}

const getFileName = (filePath) => {
  if (!filePath) return ''
  const parts = filePath.split(/[\\/]/)
  return parts[parts.length - 1]
}

const getDisplayPath = (filePath) => {
  const parts = filePath.split(/[\\/]/)
  if (parts.length > 2) {
    return '...' + parts.slice(-2, -1).join('/') + '/'
  }
  return parts.slice(0, -1).join('/') + '/'
}

onMounted(() => {
  document.addEventListener('click', handleGlobalClick)
})

onUnmounted(() => {
  document.removeEventListener('click', handleGlobalClick)
})
</script>

<style scoped>
.menu-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  background: var(--verse-bg-secondary);
  border-bottom: 1px solid var(--verse-border-light);
  padding: 0 var(--verse-spacing-md);
  height: 38px;
  user-select: none;
  font-size: 13px;
  font-weight: 400;
  box-shadow: 0 1px 0 var(--verse-border-light);
}

.menu-section {
  display: flex;
  align-items: center;
  gap: 2px;
}

.menu-item {
  position: relative;
  display: flex;
  align-items: center;
  padding: 6px 12px;
  cursor: pointer;
  border-radius: var(--verse-radius-sm);
  transition: all 0.15s ease;
  border: 1px solid transparent;
}

.menu-item:hover {
  background: var(--verse-bg-hover);
}

.menu-label {
  color: var(--verse-text-primary);
  font-weight: 500;
}

.dropdown-menu {
  position: absolute;
  top: calc(100% + 4px);
  left: 0;
  background: var(--verse-bg-secondary);
  border: 1px solid var(--verse-border-light);
  border-radius: var(--verse-radius-md);
  box-shadow: var(--verse-shadow-xl);
  min-width: 240px;
  padding: var(--verse-spacing-xs) 0;
  z-index: var(--verse-z-dropdown);
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  overflow: hidden;
}

.menu-option {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--verse-spacing-sm) var(--verse-spacing-md);
  cursor: pointer;
  color: var(--verse-text-primary);
  transition: all 0.2s ease;
  position: relative;
  font-size: 13px;
  margin: 1px var(--verse-spacing-xs);
  border-radius: var(--verse-radius-sm);
}

.menu-option:hover:not(.disabled) {
  background: var(--verse-bg-hover);
}

.menu-option.disabled {
  color: var(--verse-text-tertiary);
  cursor: not-allowed;
}

.menu-option.active {
  background: var(--verse-bg-selected);
  color: var(--verse-accent);
}

.shortcut {
  color: var(--verse-text-tertiary);
  font-size: 11px;
  font-family: 'SF Mono', Monaco, 'Cascadia Code', monospace;
}

.menu-divider {
  height: 1px;
  background: var(--verse-border-color);
  margin: var(--verse-spacing-xs) 0;
}

.recent-files {
  position: relative;
}

.arrow-icon {
  margin-left: var(--verse-spacing-sm);
  transition: transform 0.2s ease;
  color: var(--verse-text-tertiary);
}

.recent-files-submenu {
  position: absolute;
  left: 100%;
  top: 0;
  background: var(--verse-bg-secondary);
  border: 1px solid var(--verse-border-light);
  border-radius: var(--verse-radius-md);
  box-shadow: var(--verse-shadow-xl);
  min-width: 320px;
  padding: var(--verse-spacing-xs) 0;
  z-index: calc(var(--verse-z-dropdown) + 1);
  backdrop-filter: blur(10px);
  -webkit-backdrop-filter: blur(10px);
}

.recent-item {
  flex-direction: column;
  align-items: flex-start;
  padding: var(--verse-spacing-sm) var(--verse-spacing-md);
  gap: 2px;
}

.file-name {
  font-weight: 500;
  color: var(--verse-text-primary);
  font-size: 13px;
}

.file-path {
  font-size: 11px;
  color: var(--verse-text-tertiary);
  font-family: 'SF Mono', Monaco, 'Cascadia Code', monospace;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}

/* 毛玻璃效果增强 */
.dropdown-menu::before,
.recent-files-submenu::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: var(--verse-bg-primary);
  opacity: 0.8;
  z-index: -1;
  border-radius: inherit;
}
</style>

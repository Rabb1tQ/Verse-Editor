<template>
  <div class="search-panel" :class="{ visible: visible }">
    <div class="search-header">
      <h3 class="search-title">全局搜索</h3>
      <button class="close-btn" @click="$emit('close')" title="关闭搜索">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
          <path d="M19,6.41L17.59,5L12,10.59L6.41,5L5,6.41L10.59,12L5,17.59L6.41,19L12,13.41L17.59,19L19,17.59L13.41,12L19,6.41Z" />
        </svg>
      </button>
    </div>
    
    <div class="search-input-section">
      <div class="search-input-wrapper">
        <svg class="search-icon" width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
          <path d="M9.5,3A6.5,6.5 0 0,1 16,9.5C16,11.11 15.41,12.59 14.44,13.73L14.71,14H15.5L20.5,19L19,20.5L14,15.5V14.71L13.73,14.44C12.59,15.41 11.11,16 9.5,16A6.5,6.5 0 0,1 3,9.5A6.5,6.5 0 0,1 9.5,3M9.5,5C7,5 5,7 5,9.5C5,12 7,14 9.5,14C12,14 14,12 14,9.5C14,7 12,5 9.5,5Z" />
        </svg>
        <input
          ref="searchInput"
          v-model="searchQuery"
          type="text"
          placeholder="搜索文件内容..."
          class="search-input"
          @input="handleSearch"
          @keydown.enter="performSearch"
          @keydown.escape="$emit('close')"
        />
        <button v-if="searchQuery" class="clear-btn" @click="clearSearch" title="清空">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
            <path d="M19,6.41L17.59,5L12,10.59L6.41,5L5,6.41L10.59,12L5,17.59L6.41,19L12,13.41L17.59,19L19,17.59L13.41,12L19,6.41Z" />
          </svg>
        </button>
      </div>
      
      <div class="search-options">
        <label class="option-item">
          <input type="checkbox" v-model="caseSensitive" @change="performSearch">
          <span>区分大小写</span>
        </label>
        <label class="option-item">
          <input type="checkbox" v-model="wholeWord" @change="performSearch">
          <span>全词匹配</span>
        </label>
        <label class="option-item">
          <input type="checkbox" v-model="useRegex" @change="performSearch">
          <span>正则表达式</span>
        </label>
      </div>
    </div>
    
    <div class="search-results">
      <div class="results-header" v-if="searchResults.length > 0 || searching">
        <span class="results-count">
          <template v-if="searching">
            搜索中...
          </template>
          <template v-else>
            找到 {{ searchResults.length }} 个结果
          </template>
        </span>
      </div>
      
      <div class="results-list" v-if="searchResults.length > 0">
        <div
          v-for="result in searchResults"
          :key="`${result.file}-${result.line}`"
          class="result-item"
          @click="openResult(result)"
        >
          <div class="result-file">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
              <path d="M14,2H6A2,2 0 0,0 4,4V20A2,2 0 0,0 6,22H18A2,2 0 0,0 20,20V8L14,2M18,20H6V4H13V9H18V20Z" />
            </svg>
            <span class="file-name">{{ getFileName(result.file) }}</span>
            <span class="line-number">:{{ result.line }}</span>
          </div>
          <div class="result-content" v-html="result.highlightedContent"></div>
        </div>
      </div>
      
      <div class="no-results" v-else-if="searchQuery && !searching">
        <svg width="48" height="48" viewBox="0 0 24 24" fill="currentColor" opacity="0.3">
          <path d="M9.5,3A6.5,6.5 0 0,1 16,9.5C16,11.11 15.41,12.59 14.44,13.73L14.71,14H15.5L20.5,19L19,20.5L14,15.5V14.71L13.73,14.44C12.59,15.41 11.11,16 9.5,16A6.5,6.5 0 0,1 3,9.5A6.5,6.5 0 0,1 9.5,3M9.5,5C7,5 5,7 5,9.5C5,12 7,14 9.5,14C12,14 14,12 14,9.5C14,7 12,5 9.5,5Z" />
        </svg>
        <p>未找到匹配结果</p>
      </div>
      
      <div class="search-placeholder" v-else-if="!searchQuery">
        <svg width="48" height="48" viewBox="0 0 24 24" fill="currentColor" opacity="0.3">
          <path d="M9.5,3A6.5,6.5 0 0,1 16,9.5C16,11.11 15.41,12.59 14.44,13.73L14.71,14H15.5L20.5,19L19,20.5L14,15.5V14.71L13.73,14.44C12.59,15.41 11.11,16 9.5,16A6.5,6.5 0 0,1 3,9.5A6.5,6.5 0 0,1 9.5,3M9.5,5C7,5 5,7 5,9.5C5,12 7,14 9.5,14C12,14 14,12 14,9.5C14,7 12,5 9.5,5Z" />
        </svg>
        <p>输入关键词开始搜索</p>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, watch, nextTick } from 'vue'
import { readTextFile, readDir } from '@tauri-apps/plugin-fs'

const props = defineProps({
  visible: {
    type: Boolean,
    default: false
  },
  currentFolder: {
    type: String,
    default: ''
  }
})

const emit = defineEmits(['close', 'open-file'])

// 搜索状态
const searchQuery = ref('')
const searchResults = ref([])
const searching = ref(false)
const caseSensitive = ref(false)
const wholeWord = ref(false)
const useRegex = ref(false)

// 引用
const searchInput = ref(null)

// 搜索防抖
let searchTimeout = null

// 监听面板显示状态
watch(() => props.visible, (visible) => {
  if (visible) {
    nextTick(() => {
      searchInput.value?.focus()
    })
  }
})

// 处理搜索输入
const handleSearch = () => {
  if (searchTimeout) {
    clearTimeout(searchTimeout)
  }
  
  searchTimeout = setTimeout(() => {
    performSearch()
  }, 300)
}

// 执行搜索
const performSearch = async () => {
  if (!searchQuery.value.trim() || !props.currentFolder) {
    searchResults.value = []
    return
  }
  
  searching.value = true
  searchResults.value = []
  
  try {
    const files = await getAllMarkdownFiles(props.currentFolder)
    const results = []
    
    for (const filePath of files) {
      try {
        const content = await readTextFile(filePath)
        const fileResults = searchInFile(filePath, content, searchQuery.value)
        results.push(...fileResults)
      } catch (error) {
        console.error(`读取文件失败: ${filePath}`, error)
      }
    }
    
    searchResults.value = results
  } catch (error) {
    console.error('搜索失败:', error)
  } finally {
    searching.value = false
  }
}

// 获取所有 Markdown 文件
const getAllMarkdownFiles = async (folderPath) => {
  const files = []
  
  const processDir = async (dirPath) => {
    try {
      const entries = await readDir(dirPath)
      
      for (const entry of entries) {
        if (entry.isDirectory) {
          await processDir(entry.path)
        } else if (isMarkdownFile(entry.name)) {
          files.push(entry.path)
        }
      }
    } catch (error) {
      console.error(`读取目录失败: ${dirPath}`, error)
    }
  }
  
  await processDir(folderPath)
  return files
}

// 判断是否为 Markdown 文件
const isMarkdownFile = (filename) => {
  const ext = filename.toLowerCase().split('.').pop()
  return ['md', 'markdown', 'txt'].includes(ext)
}

// 在文件中搜索
const searchInFile = (filePath, content, query) => {
  const lines = content.split('\n')
  const results = []
  
  let searchPattern
  try {
    if (useRegex.value) {
      const flags = caseSensitive.value ? 'g' : 'gi'
      searchPattern = new RegExp(query, flags)
    } else {
      const escapedQuery = query.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
      const pattern = wholeWord.value ? `\\b${escapedQuery}\\b` : escapedQuery
      const flags = caseSensitive.value ? 'g' : 'gi'
      searchPattern = new RegExp(pattern, flags)
    }
  } catch (error) {
    // 正则表达式错误，使用普通文本搜索
    const flags = caseSensitive.value ? 'g' : 'gi'
    searchPattern = new RegExp(query.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'), flags)
  }
  
  lines.forEach((line, index) => {
    if (searchPattern.test(line)) {
      const highlightedContent = line.replace(searchPattern, '<mark>$&</mark>')
      results.push({
        file: filePath,
        line: index + 1,
        content: line,
        highlightedContent
      })
    }
  })
  
  return results
}

// 获取文件名
const getFileName = (filePath) => {
  const parts = filePath.split(/[\\/]/)
  return parts[parts.length - 1]
}

// 清空搜索
const clearSearch = () => {
  searchQuery.value = ''
  searchResults.value = []
}

// 打开搜索结果
const openResult = (result) => {
  emit('open-file', result.file, result.line)
}

// 暴露方法
defineExpose({
  focus: () => searchInput.value?.focus(),
  clear: clearSearch
})
</script>

<style scoped>
.search-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: #f8f9fa;
  border-right: 1px solid #e9ecef;
  transform: translateX(-100%);
  transition: transform 0.3s ease;
}

.search-panel.visible {
  transform: translateX(0);
}

.search-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-bottom: 1px solid #e9ecef;
  background: #ffffff;
}

.search-title {
  font-size: 14px;
  font-weight: 600;
  color: #495057;
  margin: 0;
}

.close-btn {
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

.close-btn:hover {
  background: #e9ecef;
  color: #495057;
}

.search-input-section {
  padding: 16px;
  border-bottom: 1px solid #e9ecef;
  background: #ffffff;
}

.search-input-wrapper {
  position: relative;
  display: flex;
  align-items: center;
}

.search-icon {
  position: absolute;
  left: 12px;
  color: #6c757d;
  pointer-events: none;
}

.search-input {
  width: 100%;
  padding: 8px 12px 8px 36px;
  border: 1px solid #ced4da;
  border-radius: 4px;
  font-size: 14px;
  background: #ffffff;
  transition: border-color 0.2s ease;
}

.search-input:focus {
  outline: none;
  border-color: #007bff;
  box-shadow: 0 0 0 2px rgba(0, 123, 255, 0.25);
}

.clear-btn {
  position: absolute;
  right: 8px;
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
  transition: all 0.2s ease;
}

.clear-btn:hover {
  background: #e9ecef;
  color: #495057;
}

.search-options {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
  margin-top: 12px;
}

.option-item {
  display: flex;
  align-items: center;
  font-size: 12px;
  color: #6c757d;
  cursor: pointer;
  user-select: none;
}

.option-item input {
  margin-right: 6px;
}

.search-results {
  flex: 1;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.results-header {
  padding: 8px 16px;
  background: #f8f9fa;
  border-bottom: 1px solid #e9ecef;
}

.results-count {
  font-size: 12px;
  color: #6c757d;
  font-weight: 500;
}

.results-list {
  flex: 1;
  overflow-y: auto;
  padding: 8px 0;
}

.result-item {
  padding: 8px 16px;
  cursor: pointer;
  border-bottom: 1px solid #f1f3f4;
  transition: background 0.2s ease;
}

.result-item:hover {
  background: #f8f9fa;
}

.result-file {
  display: flex;
  align-items: center;
  margin-bottom: 4px;
  font-size: 12px;
  color: #6c757d;
}

.result-file svg {
  margin-right: 6px;
}

.file-name {
  font-weight: 500;
}

.line-number {
  color: #007bff;
  margin-left: 4px;
}

.result-content {
  font-size: 13px;
  color: #495057;
  line-height: 1.4;
  word-break: break-word;
}

.result-content :deep(mark) {
  background: #fff3cd;
  color: #856404;
  padding: 1px 2px;
  border-radius: 2px;
}

.no-results,
.search-placeholder {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: #6c757d;
  text-align: center;
}

.no-results svg,
.search-placeholder svg {
  margin-bottom: 16px;
}

.no-results p,
.search-placeholder p {
  margin: 0;
  font-size: 14px;
}

@media (prefers-color-scheme: dark) {
  .search-panel {
    background: #2d3748;
    border-right-color: #4a5568;
  }

  .search-header {
    background: #2d3748;
    border-bottom-color: #4a5568;
  }

  .search-title {
    color: #e2e8f0;
  }

  .close-btn {
    color: #a0aec0;
  }

  .close-btn:hover {
    background: #4a5568;
    color: #f7fafc;
  }

  .search-input-section {
    background: #2d3748;
    border-bottom-color: #4a5568;
  }

  .search-icon {
    color: #a0aec0;
  }

  .search-input {
    background: #4a5568;
    border-color: #718096;
    color: #e2e8f0;
  }

  .search-input:focus {
    border-color: #3182ce;
  }

  .clear-btn {
    color: #a0aec0;
  }

  .clear-btn:hover {
    background: #718096;
    color: #f7fafc;
  }

  .option-item {
    color: #a0aec0;
  }

  .results-header {
    background: #2d3748;
    border-bottom-color: #4a5568;
  }

  .results-count {
    color: #a0aec0;
  }

  .result-item {
    border-bottom-color: #4a5568;
  }

  .result-item:hover {
    background: #4a5568;
  }

  .result-file {
    color: #a0aec0;
  }

  .line-number {
    color: #3182ce;
  }

  .result-content {
    color: #e2e8f0;
  }

  .result-content :deep(mark) {
    background: #d69e2e;
    color: #1a202c;
  }

  .no-results,
  .search-placeholder {
    color: #a0aec0;
  }
}
</style>

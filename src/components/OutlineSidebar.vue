<template>
  <div class="outline-sidebar">
    <div class="outline-header">
      <h3 class="outline-title">文档大纲</h3>
      <div class="outline-actions">
        <button class="action-btn" @click="refreshOutline" title="刷新大纲">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <path d="M17.65,6.35C16.2,4.9 14.21,4 12,4A8,8 0 0,0 4,12A8,8 0 0,0 12,20C15.73,20 18.84,17.45 19.73,14H17.65C16.83,16.33 14.61,18 12,18A6,6 0 0,1 6,12A6,6 0 0,1 12,6C13.66,6 15.14,6.69 16.22,7.78L13,11H20V4L17.65,6.35Z" />
          </svg>
        </button>
        <button class="action-btn" @click="toggleCollapseAll" :title="allCollapsed ? '展开所有' : '折叠所有'">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <path v-if="allCollapsed" d="M7.41,8.58L12,13.17L16.59,8.58L18,10L12,16L6,10L7.41,8.58Z" />
            <path v-else d="M7.41,15.41L12,10.83L16.59,15.41L18,14L12,8L6,14L7.41,15.41Z" />
          </svg>
        </button>
      </div>
    </div>
    
    <div class="outline-content" v-if="outlineItems.length > 0">
      <div class="outline-tree">
        <OutlineItem
          v-for="item in outlineItems"
          :key="item.id"
          :item="item"
          :active-id="activeHeadingId"
          @item-click="handleItemClick"
        />
      </div>
    </div>
    
    <div class="outline-empty" v-else>
      <div class="empty-content">
        <svg width="48" height="48" viewBox="0 0 24 24" fill="currentColor" opacity="0.3">
          <path d="M14,17H7V15H14M17,13H7V11H17M17,9H7V7H17M19,3H5C3.89,3 3,3.89 3,5V19A2,2 0 0,0 5,21H19A2,2 0 0,0 21,19V5C21,4.89 20.1,4 19,4Z" />
        </svg>
        <p>文档中暂无标题</p>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, watch, nextTick } from 'vue'
import OutlineItem from './OutlineItem.vue'

const props = defineProps({
  content: {
    type: String,
    default: ''
  }
})

const emit = defineEmits(['heading-click'])

const outlineItems = ref([])
const activeHeadingId = ref('')
const allCollapsed = ref(false)

// 解析 Markdown 内容生成大纲
const parseOutline = (markdown) => {
  if (!markdown) return []
  
  const lines = markdown.split('\n')
  const headings = []
  let idCounter = 0
  
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i].trim()
    const match = line.match(/^(#{1,6})\s+(.+)$/)
    
    if (match) {
      const level = match[1].length
      const text = match[2].replace(/\[([^\]]+)\]\([^)]+\)/g, '$1') // 移除链接格式
      const id = `heading-${++idCounter}`
      
      headings.push({
        id,
        level,
        text,
        line: i + 1,
        children: [],
        collapsed: allCollapsed.value
      })
    }
  }
  
  // 构建层次结构
  return buildHierarchy(headings)
}

// 构建层次结构
const buildHierarchy = (headings) => {
  const result = []
  const stack = []
  
  for (const heading of headings) {
    // 找到合适的父级
    while (stack.length > 0 && stack[stack.length - 1].level >= heading.level) {
      stack.pop()
    }
    
    if (stack.length === 0) {
      result.push(heading)
    } else {
      stack[stack.length - 1].children.push(heading)
    }
    
    stack.push(heading)
  }
  
  return result
}

// 刷新大纲
const refreshOutline = () => {
  outlineItems.value = parseOutline(props.content)
}

// 折叠/展开所有项目
const toggleCollapseAll = () => {
  allCollapsed.value = !allCollapsed.value
  toggleAllItems(outlineItems.value, allCollapsed.value)
}

// 递归切换所有项目的折叠状态
const toggleAllItems = (items, collapsed) => {
  for (const item of items) {
    item.collapsed = collapsed
    if (item.children.length > 0) {
      toggleAllItems(item.children, collapsed)
    }
  }
}

// 处理项目点击
const handleItemClick = (item) => {
  activeHeadingId.value = item.id
  emit('heading-click', item)
}

// 监听内容变化
watch(() => props.content, () => {
  refreshOutline()
}, { immediate: true })

// 暴露方法
defineExpose({
  refreshOutline,
  setActiveHeading: (id) => {
    activeHeadingId.value = id
  }
})
</script>

<style scoped>
.outline-sidebar {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--verse-bg-secondary);
}

.outline-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-bottom: 1px solid var(--verse-border-light);
  background: var(--verse-bg-tertiary);
}

.outline-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--verse-text-primary);
  margin: 0;
}

.outline-actions {
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

.action-btn:hover {
  background: var(--verse-bg-hover);
  color: var(--verse-text-primary);
}

.outline-content {
  flex: 1;
  overflow: hidden;
}

.outline-tree {
  height: 100%;
  overflow-y: auto;
  padding: 8px 0;
}

.outline-empty {
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
  margin: 0;
  font-size: 14px;
}
</style>

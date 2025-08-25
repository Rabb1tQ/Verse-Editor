<template>
  <div class="toolbar">
    <div class="toolbar-section">
      <button class="toolbar-btn" @click="$emit('new-file')" title="新建文件 (Ctrl+N)">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
          <path d="M14,2H6A2,2 0 0,0 4,4V20A2,2 0 0,0 6,22H18A2,2 0 0,0 20,20V8L14,2M18,20H6V4H13V9H18V20Z" />
        </svg>
      </button>
      <button class="toolbar-btn" @click="$emit('open-file')" title="打开文件 (Ctrl+O)">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
          <path d="M14,2H6A2,2 0 0,0 4,4V20A2,2 0 0,0 6,22H18A2,2 0 0,0 20,20V8L14,2M18,20H6V4H13V9H18V20M15,13V16H13V18H16V21H18V18H21V16H18V13H15Z" />
        </svg>
      </button>
      <button class="toolbar-btn" @click="$emit('save-file')" title="保存文件 (Ctrl+S)">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
          <path d="M15,9H5V5H15M12,19A3,3 0 0,1 9,16A3,3 0 0,1 12,13A3,3 0 0,1 15,16A3,3 0 0,1 12,19M17,3H5C3.89,3 3,3.9 3,5V19A2,2 0 0,0 5,21H19A2,2 0 0,0 21,19V7L17,3Z" />
        </svg>
      </button>
      <button class="toolbar-btn" @click="$emit('save-as')" title="另存为 (Ctrl+Shift+S)">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
          <path d="M17,3H5C3.89,3 3,3.9 3,5V19A2,2 0 0,0 5,21H19A2,2 0 0,0 21,19V7L17,3M12,19A3,3 0 0,1 9,16A3,3 0 0,1 12,13A3,3 0 0,1 15,16A3,3 0 0,1 12,19M15,9H5V5H15M17,11V15H19V17H17V19H15V17H13V15H15V13H17V11H19V9H17V11Z" />
        </svg>
      </button>
    </div>

    <div class="toolbar-section">
      <div class="divider"></div>
      <button 
        class="toolbar-btn mode-btn" 
        :class="{ active: mode === 'wysiwyg' }"
        @click="$emit('change-mode', 'wysiwyg')" 
        title="所见即所得模式"
      >
        WYSIWYG
      </button>
      <button 
        class="toolbar-btn mode-btn" 
        :class="{ active: mode === 'ir' }"
        @click="$emit('change-mode', 'ir')" 
        title="即时渲染模式"
      >
        IR
      </button>
      <button 
        class="toolbar-btn mode-btn" 
        :class="{ active: mode === 'sv' }"
        @click="$emit('change-mode', 'sv')" 
        title="分屏预览模式"
      >
        SV
      </button>
    </div>

    <div class="toolbar-section file-info">
      <span class="filename" :title="currentFile || '未命名文档'">
        {{ displayFileName }}
        <span v-if="isModified" class="modified-indicator">•</span>
      </span>
    </div>
  </div>
</template>

<script setup>
import { computed } from 'vue'

const props = defineProps({
  currentFile: {
    type: String,
    default: ''
  },
  isModified: {
    type: Boolean,
    default: false
  },
  mode: {
    type: String,
    default: 'ir'
  }
})

defineEmits([
  'new-file',
  'open-file', 
  'save-file',
  'save-as',
  'change-mode'
])

const displayFileName = computed(() => {
  if (!props.currentFile) return '未命名文档'
  const parts = props.currentFile.split(/[\\/]/)
  return parts[parts.length - 1]
})
</script>

<style scoped>
.toolbar {
  display: flex;
  align-items: center;
  background: #f8f9fa;
  border-bottom: 1px solid #e9ecef;
  padding: 8px 12px;
  min-height: 44px;
  user-select: none;
}

.toolbar-section {
  display: flex;
  align-items: center;
  gap: 4px;
}

.toolbar-section:not(:last-child) {
  margin-right: 12px;
}

.toolbar-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 6px 8px;
  border: none;
  background: transparent;
  border-radius: 4px;
  cursor: pointer;
  color: #495057;
  transition: all 0.2s ease;
  font-size: 12px;
  font-weight: 500;
}

.toolbar-btn:hover {
  background: #e9ecef;
  color: #212529;
}

.toolbar-btn:active {
  background: #dee2e6;
}

.mode-btn {
  min-width: 60px;
  font-family: 'Segoe UI', system-ui, sans-serif;
}

.mode-btn.active {
  background: #007bff;
  color: white;
}

.mode-btn.active:hover {
  background: #0056b3;
}

.divider {
  width: 1px;
  height: 20px;
  background: #dee2e6;
  margin: 0 8px;
}

.file-info {
  flex: 1;
  justify-content: flex-end;
}

.filename {
  font-size: 13px;
  color: #6c757d;
  font-weight: 500;
  max-width: 300px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.modified-indicator {
  color: #dc3545;
  font-weight: bold;
  margin-left: 4px;
}

@media (prefers-color-scheme: dark) {
  .toolbar {
    background: #2d3748;
    border-bottom-color: #4a5568;
  }

  .toolbar-btn {
    color: #e2e8f0;
  }

  .toolbar-btn:hover {
    background: #4a5568;
    color: #f7fafc;
  }

  .toolbar-btn:active {
    background: #718096;
  }

  .mode-btn.active {
    background: #3182ce;
  }

  .mode-btn.active:hover {
    background: #2c5282;
  }

  .divider {
    background: #4a5568;
  }

  .filename {
    color: #a0aec0;
  }
}
</style>

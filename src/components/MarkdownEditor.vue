<template>
  <div class="editor-container">
    <div ref="editorRef" class="vditor-editor"></div>
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted, watch } from 'vue'
import Vditor from 'vditor'
import 'vditor/dist/index.css'
import { convertFileSrc } from '@tauri-apps/api/core'

const props = defineProps({
  content: {
    type: String,
    default: ''
  },
  mode: {
    type: String,
    default: 'ir', // 'wysiwyg' | 'ir' | 'sv'
    validator: (value) => ['wysiwyg', 'ir', 'sv'].includes(value)
  },
  currentFilePath: {
    type: String,
    default: ''
  }
})

const emit = defineEmits(['update:content', 'save'])

const editorRef = ref(null)
let vditor = null

// 防抖函数，避免频繁重新初始化
const debounce = (func, wait) => {
  let timeout
  return function executedFunction(...args) {
    const later = () => {
      clearTimeout(timeout)
      func(...args)
    }
    clearTimeout(timeout)
    timeout = setTimeout(later, wait)
  }
}

// 简化的主题切换 - 不重新初始化编辑器，只更新样式
const observeThemeChange = () => {
  const observer = new MutationObserver((mutations) => {
    mutations.forEach((mutation) => {
      if (mutation.type === 'attributes' && mutation.attributeName === 'class') {
        const target = mutation.target
        if (target === document.documentElement && vditor) {
          // 只更新编辑器主题相关的CSS类，不重新初始化
          const isDark = document.documentElement.classList.contains('dark')
          const editorElement = vditor.vditor.element
          
          if (editorElement) {
            // 更新编辑器容器的主题类
            if (isDark) {
              editorElement.classList.add('vditor--dark')
              editorElement.classList.remove('vditor--light')
            } else {
              editorElement.classList.add('vditor--light')
              editorElement.classList.remove('vditor--dark')
            }
          }
        }
      }
    })
  })
  
  observer.observe(document.documentElement, {
    attributes: true,
    attributeFilter: ['class']
  })
  
  return observer
}

let themeObserver = null

onMounted(() => {
  initEditor()
  themeObserver = observeThemeChange()
})

onUnmounted(() => {
  if (vditor) {
    vditor.destroy()
    vditor = null
  }
  if (themeObserver) {
    themeObserver.disconnect()
  }
})

watch(() => props.content, (newContent) => {
  if (vditor && vditor.getValue() !== newContent) {
    vditor.setValue(newContent)
  }
})

watch(() => props.mode, (newMode) => {
  if (vditor) {
    // 重新初始化编辑器以切换模式
    vditor.destroy()
    initEditor()
  }
})

// 设置图片路径处理器
const setupImagePathHandler = () => {
  if (!vditor) return
  
  // 监听DOM变化，处理图片路径
  const observer = new MutationObserver((mutations) => {
    mutations.forEach((mutation) => {
      if (mutation.type === 'childList') {
        // 查找新添加的图片元素
        mutation.addedNodes.forEach((node) => {
          if (node.nodeType === Node.ELEMENT_NODE) {
            const images = node.querySelectorAll ? node.querySelectorAll('img') : []
            const singleImg = node.tagName === 'IMG' ? [node] : []
            
            ;[...images, ...singleImg].forEach(async (img) => {
              await processImageSrc(img)
            })
          }
        })
      }
    })
  })
  
  // 观察编辑器内容区域
  const contentAreas = [
    vditor.vditor.ir?.element,
    vditor.vditor.wysiwyg?.element,
    vditor.vditor.preview?.element
  ].filter(Boolean)
  
  contentAreas.forEach((area) => {
    if (area) {
      observer.observe(area, {
        childList: true,
        subtree: true
      })
      
      // 处理已存在的图片
      const existingImages = area.querySelectorAll('img')
      existingImages.forEach(async (img) => {
        await processImageSrc(img)
      })
    }
  })
}

// 处理单个图片的src属性
const processImageSrc = async (img) => {
  if (!img.src || img.dataset.processed) return
  
  const originalSrc = img.getAttribute('src') || img.src
  
  try {
    if (!originalSrc.startsWith('http') && 
        !originalSrc.startsWith('data:') && 
        !originalSrc.startsWith('tauri://') &&
        !originalSrc.startsWith('blob:') &&
        !originalSrc.startsWith('asset://')) {
      
      if (props.currentFilePath) {
        let fullPath
        if (originalSrc.startsWith('/') || originalSrc.startsWith('\\')) {
          // 绝对路径
          fullPath = originalSrc
        } else {
          // 相对路径，相对于当前文件
          const currentDir = props.currentFilePath.substring(0, props.currentFilePath.lastIndexOf('/'))
          // 清理路径中的 ./ 和 ../ 以及多余的斜杠
          let cleanSrc = originalSrc
            .replace(/^\.\//, '')  // 移除开头的 ./
            .replace(/\/+/g, '/')  // 合并多个斜杠
          
          // 处理 ../ 路径
          const currentDirParts = currentDir.split('/')
          const srcParts = cleanSrc.split('/')
          
          let finalDirParts = [...currentDirParts]
          for (const part of srcParts) {
            if (part === '..') {
              finalDirParts.pop()
            } else if (part !== '.' && part !== '') {
              finalDirParts.push(part)
            }
          }
          
          fullPath = finalDirParts.join('/').replace(/\\/g, '/')
        }
        
        // 使用Tauri的convertFileSrc转换为可访问的URL
        try {
          const convertedSrc = convertFileSrc(fullPath)
          img.src = convertedSrc
          img.dataset.processed = 'true'
          
          console.log('图片路径转换:', originalSrc, '->', fullPath, '->', convertedSrc)
        } catch (fileError) {
          console.warn('图片路径转换失败:', fullPath, fileError)
        }
      }
    }
  } catch (error) {
    console.warn('图片路径转换失败:', originalSrc, error)
  }
}

const initEditor = () => {
  if (!editorRef.value) return

  // 检测当前主题
  const isDark = document.documentElement.classList.contains('dark')

  vditor = new Vditor(editorRef.value, {
    height: '100%',
    mode: props.mode,
    value: props.content,
    cache: {
      enable: false
    },
    theme: isDark ? 'dark' : 'classic',
    // 完全禁用HTML清理
    sanitize: false,
    preview: {
      theme: {
        current: isDark ? 'dark' : 'light',
        path: 'https://unpkg.com/vditor/dist/css/content-theme'
      },
      hljs: {
        style: isDark ? 'github-dark' : 'github',
        enable: true,
        lineNumber: true
      },
      math: {
        inlineDigit: true
      },
      markdown: {
        sanitize: false,  // 允许HTML标签
        toc: true,
        mark: true,
        footnotes: true,
        autoSpace: true,
        // 允许HTML标签的白名单
        allowedTags: ['font', 'span', 'div', 'p', 'strong', 'em', 'u', 'del', 'code', 'pre', 'h1', 'h2', 'h3', 'h4', 'h5', 'h6', 'blockquote', 'ul', 'ol', 'li', 'a', 'img', 'br', 'hr'],
        allowedAttributes: {
          'font': ['color', 'size', 'face'],
          'span': ['style', 'class', 'color'],
          'div': ['style', 'class'],
          'p': ['style', 'class'],
          'strong': ['style', 'class'],
          'em': ['style', 'class'],
          'a': ['href', 'title', 'target'],
          'img': ['src', 'alt', 'title', 'width', 'height']
        }
      },

    },

    toolbar: [
      'headings',
      'bold',
      'italic',
      'strike',
      '|',
      'quote',
      'line',
      'code',
      'inline-code',
      '|',
      'list',
      'ordered-list',
      'check',
      '|',
      'link',
      'table',
      '|',
      'undo',
      'redo',
      '|',
      'fullscreen',
      'edit-mode'
    ],
    counter: {
      enable: false
    },
    outline: {
      enable: false  // 禁用内置大纲
    },
    options: {
      mode: 'ir',
      debugger: false,
      typewriterMode: false,
      cdn: 'https://unpkg.com/vditor',
      lang: 'zh_CN'
    },
    // 配置HTML标签白名单
    hint: {
      parse: false,
      delay: 200
    },
    // 上传和图片处理配置
    upload: {
      accept: 'image/*',
      handler: (files) => {
        // 对于本地图片，不做上传处理，直接返回路径
        return null
      }
    },
    after: () => {
      console.log('Vditor initialized')
      // 初始化时设置主题类
      const editorElement = vditor.vditor.element
      if (editorElement) {
        if (isDark) {
          editorElement.classList.add('vditor--dark')
        } else {
          editorElement.classList.add('vditor--light')
        }
      }
      
      // 为HTML标签添加样式支持
      const styleElement = document.createElement('style')
      styleElement.textContent = `
        .vditor-ir font[color], .vditor-wysiwyg font[color],
        .vditor-ir span[style*="color"], .vditor-wysiwyg span[style*="color"] {
          color: inherit !important;
        }
        .vditor-ir font[color="red"], .vditor-wysiwyg font[color="red"],
        .vditor-ir span[style*="color:red"], .vditor-wysiwyg span[style*="color:red"],
        .vditor-ir span[style*="color: red"], .vditor-wysiwyg span[style*="color: red"] {
          color: red !important;
        }
        .vditor-ir font[color="blue"], .vditor-wysiwyg font[color="blue"],
        .vditor-ir span[style*="color:blue"], .vditor-wysiwyg span[style*="color:blue"],
        .vditor-ir span[style*="color: blue"], .vditor-wysiwyg span[style*="color: blue"] {
          color: blue !important;
        }
        .vditor-ir font[color="green"], .vditor-wysiwyg font[color="green"],
        .vditor-ir span[style*="color:green"], .vditor-wysiwyg span[style*="color:green"],
        .vditor-ir span[style*="color: green"], .vditor-wysiwyg span[style*="color: green"] {
          color: green !important;
        }
        .vditor-ir font[color="orange"], .vditor-wysiwyg font[color="orange"],
        .vditor-ir span[style*="color:orange"], .vditor-wysiwyg span[style*="color:orange"],
        .vditor-ir span[style*="color: orange"], .vditor-wysiwyg span[style*="color: orange"] {
          color: orange !important;
        }
        .vditor-ir font[color="purple"], .vditor-wysiwyg font[color="purple"],
        .vditor-ir span[style*="color:purple"], .vditor-wysiwyg span[style*="color:purple"],
        .vditor-ir span[style*="color: purple"], .vditor-wysiwyg span[style*="color: purple"] {
          color: purple !important;
        }
        /* 支持16进制颜色 */
        .vditor-ir span[style*="#"], .vditor-wysiwyg span[style*="#"] {
          color: inherit !important;
        }
      `
      document.head.appendChild(styleElement)
      
      // 处理图片路径转换
      setupImagePathHandler()
    },
    input: (value) => {
      emit('update:content', value)
      // 延迟处理图片，确保DOM已更新
      setTimeout(() => {
        if (vditor) {
          const contentAreas = [
            vditor.vditor.ir?.element,
            vditor.vditor.wysiwyg?.element,
            vditor.vditor.preview?.element
          ].filter(Boolean)
          
          contentAreas.forEach((area) => {
            if (area) {
              const images = area.querySelectorAll('img')
              images.forEach(async (img) => {
                await processImageSrc(img)
              })
            }
          })
        }
      }, 100)
    },
    ctrlEnter: (value) => {
      emit('save', value)
    }
  })
}

// 滚动到指定行
const scrollToLine = (line) => {
  if (!vditor) return
  
  try {
    // 获取编辑器内容
    const content = vditor.getValue()
    const lines = content.split('\n')
    
    // 计算目标行的字符位置
    let charPosition = 0
    for (let i = 0; i < Math.min(line - 1, lines.length - 1); i++) {
      charPosition += lines[i].length + 1 // +1 for the newline character
    }
    
    // 根据编辑器模式进行滚动
    if (vditor.vditor.currentMode === 'wysiwyg') {
      // WYSIWYG 模式
      const wysiwygElement = vditor.vditor.wysiwyg.element
      if (wysiwygElement) {
        const headings = wysiwygElement.querySelectorAll('h1, h2, h3, h4, h5, h6')
        if (headings[line - 1]) {
          headings[line - 1].scrollIntoView({ behavior: 'smooth', block: 'start' })
        }
      }
    } else {
      // IR 或 SV 模式
      const editorElement = vditor.vditor.ir?.element || vditor.vditor.sv?.element
      if (editorElement) {
        // 尝试滚动到对应行
        const lineHeight = 24 // 估算行高
        const scrollTop = (line - 1) * lineHeight
        editorElement.scrollTop = scrollTop
      }
    }
  } catch (error) {
    console.error('滚动到指定行失败:', error)
  }
}

// 暴露方法给父组件
defineExpose({
  getValue: () => vditor?.getValue() || '',
  setValue: (value) => vditor?.setValue(value),
  focus: () => vditor?.focus(),
  blur: () => vditor?.blur(),
  disabled: () => vditor?.disabled(),
  enable: () => vditor?.enable(),
  getHTML: () => vditor?.getHTML() || '',
  scrollToLine
})
</script>

<style scoped>
.editor-container {
  width: 100%;
  height: 100%;
  overflow: hidden;
}

.vditor-editor {
  height: 100%;
}

/* 确保HTML标签在编辑器中正确渲染 */
.vditor-editor :deep(.vditor-ir) font,
.vditor-editor :deep(.vditor-wysiwyg) font {
  color: inherit !important;
}

.vditor-editor :deep(.vditor-ir) font[color],
.vditor-editor :deep(.vditor-wysiwyg) font[color] {
  color: attr(color) !important;
}

/* 支持常见的HTML颜色标签 */
.vditor-editor :deep(.vditor-ir) font[color="red"],
.vditor-editor :deep(.vditor-wysiwyg) font[color="red"] {
  color: red !important;
}

.vditor-editor :deep(.vditor-ir) font[color="blue"],
.vditor-editor :deep(.vditor-wysiwyg) font[color="blue"] {
  color: blue !important;
}

.vditor-editor :deep(.vditor-ir) font[color="green"],
.vditor-editor :deep(.vditor-wysiwyg) font[color="green"] {
  color: green !important;
}

.vditor-editor :deep(.vditor-ir) font[color="orange"],
.vditor-editor :deep(.vditor-wysiwyg) font[color="orange"] {
  color: orange !important;
}

.vditor-editor :deep(.vditor-ir) font[color="purple"],
.vditor-editor :deep(.vditor-wysiwyg) font[color="purple"] {
  color: purple !important;
}
</style>

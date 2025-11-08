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
  if (!img.src) return
  
  const originalSrc = img.getAttribute('src') || img.src
  
  // 检查是否已经是转换后的 asset URL，如果是则跳过
  if (originalSrc.startsWith('http://asset.localhost/') || 
      originalSrc.startsWith('https://asset.localhost/') ||
      originalSrc.startsWith('asset://')) {
    return
  }
  
  try {
    // 只处理需要转换的本地路径
    if (!originalSrc.startsWith('http') && 
        !originalSrc.startsWith('data:') && 
        !originalSrc.startsWith('tauri://') &&
        !originalSrc.startsWith('blob:')) {
      
      if (props.currentFilePath) {
        let fullPath
        if (originalSrc.startsWith('/') || originalSrc.startsWith('\\')) {
          // 绝对路径
          fullPath = originalSrc
        } else {
          // 相对路径，相对于当前文件
          // 标准化路径分隔符，统一使用正斜杠
          const normalizedFilePath = props.currentFilePath.replace(/\\/g, '/')
          const lastSlashIndex = Math.max(normalizedFilePath.lastIndexOf('/'), normalizedFilePath.lastIndexOf('\\'))
          const currentDir = normalizedFilePath.substring(0, lastSlashIndex)
          
          // 清理路径中的 ./ 和 ../ 以及多余的斜杠
          let cleanSrc = originalSrc
            .replace(/\\/g, '/')   // 统一使用正斜杠
            .replace(/^\.\//, '')  // 移除开头的 ./
            .replace(/\/+/g, '/')  // 合并多个斜杠
          
          // 处理 ../ 路径
          const currentDirParts = currentDir.split('/').filter(part => part !== '')
          const srcParts = cleanSrc.split('/').filter(part => part !== '')
          
          let finalDirParts = [...currentDirParts]
          for (const part of srcParts) {
            if (part === '..') {
              if (finalDirParts.length > 0) {
                finalDirParts.pop()
              }
            } else if (part !== '.' && part !== '') {
              finalDirParts.push(part)
            }
          }
          
          fullPath = finalDirParts.join('/')
          // 确保在Windows上使用正确的路径格式
          if (window.__TAURI__) {
            fullPath = fullPath.replace(/\//g, '\\')
          }
        }
        
        // 使用Tauri的convertFileSrc转换为可访问的URL
        try {
          const convertedSrc = convertFileSrc(fullPath)
          img.src = convertedSrc
          // 存储原始路径和完整路径，用于后续检查文件是否存在
          img.dataset.originalPath = originalSrc
          img.dataset.fullPath = fullPath
          
          // 添加图片加载错误处理
          img.onerror = () => {
            console.warn('图片加载失败:', fullPath)
            // 为失败的图片添加一个样式类
            img.classList.add('image-load-error')
            img.title = `图片加载失败: ${originalSrc}`
            // 设置一个占位符，让用户知道图片已损坏
            img.alt = `❌ 图片不存在: ${originalSrc}`
          }
          
          // 图片成功加载时移除错误样式
          img.onload = () => {
            img.classList.remove('image-load-error')
            // 恢复原始 alt 文本
            if (img.alt && img.alt.startsWith('❌')) {
              img.alt = originalSrc
            }
          }
          
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
        current: isDark ? 'dark' : 'light'
      },
      hljs: {
        style: isDark ? 'github-dark' : 'github',
        enable: true,
        lineNumber: false,
        langs: ['javascript', 'typescript', 'python', 'java', 'cpp', 'c', 'css', 'html', 'xml', 'json', 'yaml', 'markdown', 'bash', 'shell', 'sql', 'php', 'go', 'rust', 'vue', 'jsx']
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
    lang: 'zh_CN',
    debugger: false,
    typewriterMode: false,
    cdn: '/vditor',  // 使用本地资源，支持离线环境
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
        
        /* 修复代码块重复渲染问题 */
        .vditor-reset pre {
          position: relative;
          background: #f6f8fa;
          border-radius: 6px;
          padding: 16px;
          overflow-x: auto;
        }
        
        /* 确保代码块内容正常显示 */
        .vditor-reset pre code {
          display: block !important;
          background: transparent !important;
          padding: 0 !important;
          border: none !important;
          border-radius: 0 !important;
          font-family: 'SFMono-Regular', Consolas, 'Liberation Mono', Menlo, monospace;
          font-size: 85%;
          line-height: 1.45;
        }
        
        /* 只隐藏明确重复的代码元素 */
        .vditor-reset pre code.hljs + code.hljs {
          display: none !important;
        }
        
        /* 修复深色主题下的代码块背景 */
        .dark .vditor-reset pre {
          background: #161b22;
        }
        
        /* 确保代码高亮颜色正常 */
        .vditor-reset pre code .hljs-keyword,
        .vditor-reset pre code .hljs-selector-tag,
        .vditor-reset pre code .hljs-title,
        .vditor-reset pre code .hljs-section,
        .vditor-reset pre code .hljs-doctag,
        .vditor-reset pre code .hljs-name,
        .vditor-reset pre code .hljs-strong {
          font-weight: bold;
        }
      `
      document.head.appendChild(styleElement)
      
      // 处理图片路径转换
      setupImagePathHandler()
      
      // 修复代码块重复渲染问题
      const fixDuplicateCodeBlocks = () => {
        const contentAreas = [
          vditor.vditor.ir?.element,
          vditor.vditor.wysiwyg?.element,
          vditor.vditor.preview?.element
        ].filter(Boolean)
        
        contentAreas.forEach((area) => {
          if (area) {
            const preElements = area.querySelectorAll('pre')
            preElements.forEach((pre) => {
              const codeElements = pre.querySelectorAll('code')
              // 如果有多个相同类名的code元素，移除重复的
              if (codeElements.length > 1) {
                const seen = new Set()
                for (let i = 0; i < codeElements.length; i++) {
                  const code = codeElements[i]
                  const content = code.textContent || code.innerText
                  const className = code.className
                  const key = `${className}-${content.substring(0, 100)}` // 使用前100个字符作为标识
                  
                  if (seen.has(key)) {
                    // 这是重复的元素，移除它
                    code.remove()
                  } else {
                    seen.add(key)
                  }
                }
              }
            })
          }
        })
      }
      
      // 立即执行一次
      fixDuplicateCodeBlocks()
      
      // 监听内容变化
      const observer = new MutationObserver(() => {
        setTimeout(fixDuplicateCodeBlocks, 100)
      })
      
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
        }
      })
    },
    input: (value) => {
      emit('update:content', value)
      // 延迟处理图片和代码块，确保DOM已更新
      setTimeout(() => {
        if (vditor) {
          const contentAreas = [
            vditor.vditor.ir?.element,
            vditor.vditor.wysiwyg?.element,
            vditor.vditor.preview?.element
          ].filter(Boolean)
          
          contentAreas.forEach((area) => {
            if (area) {
              // 处理图片
              const images = area.querySelectorAll('img')
              images.forEach(async (img) => {
                await processImageSrc(img)
              })
              
              // 修复重复的代码块
              const preElements = area.querySelectorAll('pre')
              preElements.forEach((pre) => {
                const codeElements = pre.querySelectorAll('code')
                if (codeElements.length > 1) {
                  const seen = new Set()
                  for (let i = 0; i < codeElements.length; i++) {
                    const code = codeElements[i]
                    const content = code.textContent || code.innerText
                    const className = code.className
                    const key = `${className}-${content.substring(0, 100)}`
                    
                    if (seen.has(key)) {
                      code.remove()
                    } else {
                      seen.add(key)
                    }
                  }
                }
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

// 刷新所有图片
const refreshImages = () => {
  if (!vditor) return
  
  console.log('🔄 开始刷新所有图片...')
  
  const contentAreas = [
    vditor.vditor.ir?.element,
    vditor.vditor.wysiwyg?.element,
    vditor.vditor.preview?.element
  ].filter(Boolean)
  
  let imageCount = 0
  
  contentAreas.forEach((area) => {
    if (area) {
      const images = area.querySelectorAll('img')
      images.forEach((img) => {
        // 强制重新加载图片，添加时间戳破坏缓存
        const currentSrc = img.src
        if (currentSrc && !currentSrc.startsWith('data:')) {
          imageCount++
          
          // 获取原始路径
          const originalPath = img.dataset.originalPath || ''
          const fullPath = img.dataset.fullPath || ''
          
          // 移除旧的时间戳参数（如果有）
          const baseUrl = currentSrc.split('?')[0]
          // 添加新的时间戳参数强制刷新
          const timestamp = new Date().getTime()
          const newSrc = `${baseUrl}?t=${timestamp}`
          
          console.log(`  📷 [${imageCount}] 刷新图片:`, originalPath || baseUrl)
          
          // 重新绑定错误和加载处理器
          img.onerror = () => {
            console.warn(`  ❌ 图片加载失败:`, fullPath || originalPath)
            img.classList.add('image-load-error')
            img.title = `图片加载失败: ${originalPath}`
            img.alt = `❌ 图片不存在: ${originalPath}`
          }
          
          img.onload = () => {
            console.log(`  ✅ 图片加载成功:`, originalPath || baseUrl)
            img.classList.remove('image-load-error')
            if (img.alt && img.alt.startsWith('❌')) {
              img.alt = originalPath
            }
          }
          
          // 设置新的 src 触发重新加载
          img.src = newSrc
        }
      })
    }
  })
  
  console.log(`✅ 已触发 ${imageCount} 张图片刷新`)
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
  scrollToLine,
  refreshImages
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

/* 图片加载失败的样式 */
.vditor-editor :deep(img.image-load-error) {
  border: 2px dashed #f56c6c !important;
  background: #fef0f0 !important;
  padding: 8px !important;
  opacity: 1 !important;
  min-width: 200px;
  min-height: 100px;
  display: inline-block !important;
  position: relative;
  font-size: 14px;
  color: #f56c6c;
}

.vditor-editor :deep(.vditor-ir img.image-load-error),
.vditor-editor :deep(.vditor-wysiwyg img.image-load-error) {
  border: 2px dashed #f56c6c !important;
  background: #fef0f0 !important;
  padding: 4px !important;
  opacity: 1 !important;
  min-width: 200px;
  min-height: 100px;
  display: inline-block !important;
}

/* 深色模式下的错误样式 */
.dark .vditor-editor :deep(img.image-load-error),
.dark .vditor-editor :deep(.vditor-ir img.image-load-error),
.dark .vditor-editor :deep(.vditor-wysiwyg img.image-load-error) {
  background: #442222 !important;
  border-color: #f56c6c !important;
  color: #f56c6c;
}
</style>

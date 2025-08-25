<template>
  <div class="outline-item">
    <div 
      class="item-content"
      :class="{ 
        'active': item.id === activeId,
        [`level-${item.level}`]: true
      }"
      @click="handleClick"
    >
      <div class="item-expand" v-if="item.children.length > 0" @click.stop="toggleCollapsed">
        <svg width="12" height="12" viewBox="0 0 24 24" fill="currentColor" 
             :style="{ transform: item.collapsed ? 'rotate(0deg)' : 'rotate(90deg)' }">
          <path d="M8.59,16.58L13.17,12L8.59,7.41L10,6L16,12L10,18L8.59,16.58Z" />
        </svg>
      </div>
      
      <div class="item-icon">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
          <path v-if="item.level === 1" d="M3,4H7V8H3V4M9,5V7H21V5H9M3,10H7V14H3V10M9,11V13H21V11H9M3,16H7V20H3V16M9,17V19H21V17H9" />
          <path v-else-if="item.level === 2" d="M3,4H7V8H3V4M9,5V7H21V5H9M3,10H7V14H3V10M9,11V13H21V11H9" />
          <path v-else-if="item.level === 3" d="M3,4H7V8H3V4M9,5V7H21V5H9" />
          <path v-else d="M9,5V7H21V5H9" />
        </svg>
      </div>
      
      <span class="item-text" :title="item.text">{{ item.text }}</span>
      
      <span class="item-line">{{ item.line }}</span>
    </div>
    
    <div v-if="item.children.length > 0 && !item.collapsed" class="item-children">
      <OutlineItem
        v-for="child in item.children"
        :key="child.id"
        :item="child"
        :active-id="activeId"
        @item-click="$emit('item-click', $event)"
      />
    </div>
  </div>
</template>

<script setup>
const props = defineProps({
  item: {
    type: Object,
    required: true
  },
  activeId: {
    type: String,
    default: ''
  }
})

const emit = defineEmits(['item-click'])

const handleClick = () => {
  emit('item-click', props.item)
}

const toggleCollapsed = () => {
  props.item.collapsed = !props.item.collapsed
}
</script>

<style scoped>
.outline-item {
  user-select: none;
}

.item-content {
  display: flex;
  align-items: center;
  padding: 4px 8px;
  cursor: pointer;
  border-radius: 4px;
  margin: 1px 8px;
  transition: all 0.2s ease;
  position: relative;
  min-height: 28px;
}

.item-content:hover {
  background: rgba(0, 123, 255, 0.1);
}

.item-content.active {
  background: #007bff;
  color: white;
}

/* 不同级别的缩进 */
.item-content.level-1 {
  padding-left: 8px;
  font-weight: 600;
}

.item-content.level-2 {
  padding-left: 24px;
  font-weight: 500;
}

.item-content.level-3 {
  padding-left: 40px;
}

.item-content.level-4 {
  padding-left: 56px;
}

.item-content.level-5 {
  padding-left: 72px;
}

.item-content.level-6 {
  padding-left: 88px;
}

.item-expand {
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

.item-expand:hover {
  background: rgba(0, 0, 0, 0.1);
}

.item-expand svg {
  transition: transform 0.2s ease;
}

.item-icon {
  display: flex;
  align-items: center;
  margin-right: 8px;
  color: #6c757d;
  flex-shrink: 0;
}

.item-content.active .item-icon {
  color: white;
}

.item-text {
  flex: 1;
  font-size: 13px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  line-height: 1.4;
}

.item-line {
  font-size: 11px;
  color: #6c757d;
  opacity: 0.7;
  margin-left: 8px;
  flex-shrink: 0;
}

.item-content.active .item-line {
  color: rgba(255, 255, 255, 0.8);
}

.item-children {
  /* 子项目不需要额外的缩进，因为已经在 level 类中处理 */
}

/* 不同级别的字体大小 */
.level-1 .item-text {
  font-size: 14px;
}

.level-2 .item-text {
  font-size: 13px;
}

.level-3 .item-text,
.level-4 .item-text,
.level-5 .item-text,
.level-6 .item-text {
  font-size: 12px;
}

@media (prefers-color-scheme: dark) {
  .item-content:hover {
    background: rgba(59, 130, 246, 0.2);
  }

  .item-content.active {
    background: #3b82f6;
  }

  .item-expand:hover {
    background: rgba(255, 255, 255, 0.1);
  }

  .item-icon {
    color: #a0aec0;
  }

  .item-line {
    color: #a0aec0;
  }
}
</style>

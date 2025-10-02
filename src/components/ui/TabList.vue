<template>
  <div role="tablist" class="tabs tabs-lift">
    <input
      v-for="(tab, i) in tabs"
      :key="tab.id"
      type="radio"
      name="tabs"
      role="tab"
      class="tab truncate min-w-[100px] max-w-[200px]"
      :aria-label="tab.label"
      :checked="i === activeIndex"
      @change="$emit('select', i)"
      @mousedown="(e) => handleMouseDown(e, tab)"
      @dblclick="() => handleRename(tab)"
    />
    <div class="tab" @click="$emit('create')">
      <svg
        xmlns="http://www.w3.org/2000/svg"
        fill="none"
        viewBox="0 0 24 24"
        stroke-width="1.5"
        stroke="currentColor"
        class="size-4"
      >
        <path
          stroke-linecap="round"
          stroke-linejoin="round"
          d="M12 4.5v15m7.5-7.5h-15"
        />
      </svg>
    </div>
  </div>
</template>

<script setup lang="ts">
const props = defineProps<{
  tabs: Tab[]
  activeIndex: number
}>()

const emit = defineEmits<{
  create: []
  remove: [id: string]
  rename: [id: string, newName: string]
  select: [index: number]
}>()

function handleMouseDown(event: MouseEvent, tab: Tab) {
  if (event.button === 1) {
    event.preventDefault()
    emit('remove', tab.id)
  }
}

function handleRename(tab: Tab) {
  const newName = prompt('Enter new name:')
  if (newName) {
    emit('rename', tab.id, newName)
  }
}
</script>

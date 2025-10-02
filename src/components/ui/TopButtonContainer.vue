<template>
  <nav class="menu">
    <button class="menu-item">File</button>
    <button class="menu-item">Edit</button>
    <button class="menu-item">View</button>
    <button class="menu-item">Playback</button>
    <button class="menu-item">Library</button>
    <button class="menu-item">Help</button>
  </nav>
</template>

<style scoped>
.menu {
  display: flex;
  gap: 6px;
}

.menu-item {
  background: transparent;
  color: var(--text);
  border: 0;
  padding: 2px 4px;
  font-size: 0.9rem;
  line-height: 1rem;
  cursor: default;
}

.menu-item:hover,
.menu-item:focus {
  color: #ffffff;
}
</style>
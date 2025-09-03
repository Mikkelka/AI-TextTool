<script setup lang="ts">
  // Minimal App.vue - This window is hidden and serves as the main Tauri entry point
  // The actual UI is handled by popup windows, chat windows, and tray interactions
</script>

<template>
  <!-- Hidden main window - UI is handled by popups and tray -->
  <div class="hidden-main">
    <p>AI TextTool is running in the background. Access it via:</p>
    <ul>
      <li>Global hotkey: Ctrl+Space (with selected text)</li>
      <li>System tray icon (right-click for menu)</li>
    </ul>
  </div>
</template>

<style scoped>
  .hidden-main {
    padding: 20px;
    text-align: center;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    color: #666;
  }

  .hidden-main ul {
    text-align: left;
    display: inline-block;
    margin-top: 16px;
  }

  .hidden-main li {
    margin: 8px 0;
  }
</style>

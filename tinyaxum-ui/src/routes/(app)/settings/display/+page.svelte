<script lang="ts">
  import ContentSection from '$lib/components/settings/ContentSection.svelte'
</script>

<ContentSection title="显示" desc="开启或关闭项目以控制应用中显示的内容。">
  <div class="space-y-6">
    <div class="space-y-4">
      <h4 class="text-sm font-medium">侧边栏</h4>
      <div class="space-y-2">
        <label class="flex items-center space-x-2">
          <input
            type="checkbox"
            checked
            class="h-4 w-4 rounded border-gray-300"
          />
          <span class="text-sm">显示团队标签</span>
        </label>
      </div>
      <div class="space-y-2">
        <label class="flex items-center space-x-2">
          <input
            type="checkbox"
            checked
            class="h-4 w-4 rounded border-gray-300"
          />
          <span class="text-sm">显示用户头像</span>
        </label>
      </div>
    </div>

    <div class="space-y-4">
      <h4 class="text-sm font-medium">仪表板</h4>
      <div class="space-y-2">
        <label class="flex items-center space-x-2">
          <input
            type="checkbox"
            checked
            class="h-4 w-4 rounded border-gray-300"
          />
          <span class="text-sm">显示分析卡片</span>
        </label>
      </div>
      <div class="space-y-2">
        <label class="flex items-center space-x-2">
          <input type="checkbox" class="h-4 w-4 rounded border-gray-300" />
          <span class="text-sm">紧凑视图</span>
        </label>
      </div>
    </div>

    <button
      class="bg-primary text-primary-foreground hover:bg-primary/90 inline-flex h-9 items-center justify-center rounded-md px-4 py-2 text-sm font-medium shadow transition-colors"
    >
      更新显示
    </button>
  </div>
</ContentSection>

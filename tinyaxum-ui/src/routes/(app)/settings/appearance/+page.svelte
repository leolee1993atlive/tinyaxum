<script lang="ts">
  import ContentSection from '$lib/components/settings/ContentSection.svelte'
  import { useTheme } from '$lib/context/theme.svelte.js'
  import { COLOR_THEMES } from '$lib/themes/registry.js'

  const theme = useTheme()
</script>

<ContentSection title="外观" desc="自定义应用的外观。">
  <div class="space-y-6">
    <!-- 颜色主题 -->
    <div class="space-y-4">
      <h4 class="text-sm font-medium">颜色主题</h4>
      <p class="text-muted-foreground text-sm">选择您偏好的颜色主题。</p>
      <div class="flex items-center space-x-4">
        {#each COLOR_THEMES as ct}
          <label class="flex items-center space-x-2">
            <input
              type="radio"
              name="colorTheme"
              value={ct.name}
              checked={theme.colorTheme === ct.name}
              onchange={() => theme.setColorTheme(ct.name)}
              class="h-4 w-4 border-gray-300"
            />
            <span class="text-sm">{ct.label}</span>
          </label>
        {/each}
      </div>
    </div>

    <!-- 外观模式 -->
    <div class="space-y-4">
      <h4 class="text-sm font-medium">外观模式</h4>
      <p class="text-muted-foreground text-sm">选择您偏好的显示模式。</p>
      <div class="flex items-center space-x-4">
        {#each [{ value: 'light', label: '浅色' }, { value: 'dark', label: '深色' }, { value: 'system', label: '跟随系统' }] as mode}
          <label class="flex items-center space-x-2">
            <input
              type="radio"
              name="themeMode"
              value={mode.value}
              checked={theme.theme === mode.value}
              onchange={() =>
                theme.setTheme(mode.value as 'light' | 'dark' | 'system')}
              class="h-4 w-4 border-gray-300"
            />
            <span class="text-sm">{mode.label}</span>
          </label>
        {/each}
      </div>
    </div>
  </div>
</ContentSection>

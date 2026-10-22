<script lang="ts">
  import ContentSection from '#lib/components/settings/ContentSection.svelte'
</script>

<ContentSection title="通知" desc="配置您接收通知的方式。">
  <div class="space-y-6">
    <div class="space-y-4">
      <h4 class="text-sm font-medium">邮件通知</h4>
      <div class="space-y-2">
        <label class="flex items-center space-x-2">
          <input
            type="checkbox"
            checked
            class="h-4 w-4 rounded border-gray-300"
          />
          <span class="text-sm">通信邮件</span>
        </label>
        <p class="text-muted-foreground text-xs">接收关于您账户活动的邮件。</p>
      </div>
      <div class="space-y-2">
        <label class="flex items-center space-x-2">
          <input type="checkbox" class="h-4 w-4 rounded border-gray-300" />
          <span class="text-sm">营销邮件</span>
        </label>
        <p class="text-muted-foreground text-xs">
          接收关于新产品、功能等的邮件。
        </p>
      </div>
      <div class="space-y-2">
        <label class="flex items-center space-x-2">
          <input type="checkbox" class="h-4 w-4 rounded border-gray-300" />
          <span class="text-sm">社交邮件</span>
        </label>
        <p class="text-muted-foreground text-xs">
          接收好友请求、关注等的邮件。
        </p>
      </div>
    </div>

    <button
      class="bg-primary text-primary-foreground hover:bg-primary/90 inline-flex h-9 items-center justify-center rounded-md px-4 py-2 text-sm font-medium shadow transition-colors"
    >
      更新通知
    </button>
  </div>
</ContentSection>

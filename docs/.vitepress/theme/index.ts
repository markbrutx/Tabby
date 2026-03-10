import DefaultTheme from 'vitepress/theme'
import type { Theme } from 'vitepress'
import DownloadSection from './DownloadSection.vue'
import './custom.css'

export default {
  extends: DefaultTheme,
  enhanceApp({ app }) {
    app.component('DownloadSection', DownloadSection)
  },
} satisfies Theme

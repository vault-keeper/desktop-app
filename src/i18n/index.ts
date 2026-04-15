import { createI18n } from 'vue-i18n'
import zhCN from './locales/zh-CN'
import enUS from './locales/en-US'

const LANG_KEY = 'vaultkeeper-lang'

function getDefaultLocale(): string {
  const saved = localStorage.getItem(LANG_KEY)
  if (saved === 'zh-CN' || saved === 'en-US') return saved
  const browser = navigator.language
  if (browser.startsWith('zh')) return 'zh-CN'
  return 'en-US'
}

export const i18n = createI18n({
  legacy: false,
  locale: getDefaultLocale(),
  fallbackLocale: 'zh-CN',
  messages: {
    'zh-CN': zhCN,
    'en-US': enUS,
  },
})

export function setLocale(lang: 'zh-CN' | 'en-US') {
  i18n.global.locale.value = lang
  localStorage.setItem(LANG_KEY, lang)
}

export type SupportedLocale = 'zh-CN' | 'en-US'

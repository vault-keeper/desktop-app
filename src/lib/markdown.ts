// Shared markdown renderer used by both the editor preview pane and any
// read-only views (note / report detail).  Keeping the configuration here
// guarantees consistent behaviour: highlight.js syntax highlighting,
// GFM-style task lists and the same set of features everywhere.

import MarkdownIt from 'markdown-it'
import hljs from './hljs'

function escapeHtml(s: string): string {
  return s
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;')
}

export const md = new MarkdownIt({
  breaks: true,
  linkify: true,
  html: false,
  // Returning a full <pre>...</pre> string makes markdown-it skip its own
  // wrapping (see fence rule), so the `hljs` class lands on <code>.
  highlight(str: string, lang: string): string {
    if (lang && hljs.getLanguage(lang)) {
      try {
        const out = hljs.highlight(str, { language: lang, ignoreIllegals: true }).value
        return `<pre><code class="hljs language-${lang}">${out}</code></pre>`
      } catch {
        // fall through to plain rendering below
      }
    }
    return `<pre><code class="hljs">${escapeHtml(str)}</code></pre>`
  },
})

/**
 * Render markdown to HTML with task-list checkbox post-processing.
 *
 * Markdown-it renders `- [ ] foo` as `<li>[ ] foo</li>` because `[ ]` is
 * just text — we rewrite that leading literal into a real (read-only)
 * checkbox so the preview matches the GFM rendering everyone expects.
 */
export function renderMarkdown(src: string): string {
  let html = md.render(src || '')
  html = html
    .replace(/<li>\s*\[ \]\s+/g, '<li class="task-list-item"><input type="checkbox" disabled class="task-list-checkbox"> ')
    .replace(/<li>\s*\[[xX]\]\s+/g, '<li class="task-list-item"><input type="checkbox" checked disabled class="task-list-checkbox"> ')
  return html
}

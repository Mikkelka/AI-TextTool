import { marked } from 'marked'
import DOMPurify from 'dompurify'

// Configure marked for GitHub Flavored Markdown
marked.setOptions({
  breaks: true, // Convert '\n' to <br>
  gfm: true // Enable GitHub Flavored Markdown
})

function escapeHtml(text: string): string {
  const div = document.createElement('div')
  div.textContent = text
  return div.innerHTML
}

/**
 * Render markdown text to safe HTML
 * @param markdown The markdown text to render
 * @returns Sanitized HTML string
 */
export function renderMarkdown(markdown: string): string {
  try {
    // Convert markdown to HTML
    let html = marked.parse(markdown) as string

    // Add custom CSS classes to elements
    html = addCustomClasses(html)

    // Sanitize HTML to prevent XSS attacks
    const sanitized = DOMPurify.sanitize(html, {
      ALLOWED_TAGS: [
        'h1',
        'h2',
        'h3',
        'h4',
        'h5',
        'h6',
        'p',
        'br',
        'hr',
        'strong',
        'em',
        'b',
        'i',
        'u',
        's',
        'del',
        'ul',
        'ol',
        'li',
        'blockquote',
        'pre',
        'code',
        'table',
        'thead',
        'tbody',
        'tr',
        'th',
        'td',
        'a',
        'div',
        'span',
        'button'
      ],
      ALLOWED_ATTR: ['href', 'target', 'rel', 'class', 'data-copy-code', 'colspan', 'rowspan'],
      ALLOWED_URI_REGEXP:
        /^(?:(?:(?:f|ht)tps?|mailto|tel|callto|cid|xmpp|xxx):|[^a-z]|[a-z+.-]+(?:[^a-z+.-:]|$))/i
    })

    return sanitized
  } catch (error) {
    console.error('Error rendering markdown:', error)
    // Fallback to escaped plain text
    return escapeHtml(markdown)
  }
}

function addCustomClasses(html: string): string {
  // Add CSS classes to elements for styling
  html = html.replace(/<h([1-6])>/g, '<h$1 class="markdown-heading markdown-h$1">')
  html = html.replace(/<table>/g, '<div class="table-wrapper"><table class="markdown-table">')
  html = html.replace(/<\/table>/g, '</table></div>')
  html = html.replace(/<blockquote>/g, '<blockquote class="markdown-blockquote">')
  html = html.replace(/<ul>/g, '<ul class="markdown-list">')
  html = html.replace(/<ol>/g, '<ol class="markdown-list">')

  // Add copy button to code blocks
  html = html.replace(/<pre><code([^>]*)>([\s\S]*?)<\/code><\/pre>/g, (_match, attrs, content) => {
    return `<pre class="code-block"><code${attrs}>${content}</code><button class="copy-code-btn" data-copy-code>📋</button></pre>`
  })

  return html
}

// Event handler for copy code buttons
const handleCopyCodeClick = (event: Event) => {
  const target = event.target as HTMLElement
  if (target.classList.contains('copy-code-btn') || target.hasAttribute('data-copy-code')) {
    const codeBlock = target.parentElement?.querySelector('code')
    if (codeBlock) {
      navigator.clipboard
        .writeText(codeBlock.textContent || '')
        .then(() => {
          // Brief visual feedback
          const originalText = target.textContent
          target.textContent = '✅'
          setTimeout(() => {
            target.textContent = originalText
          }, 1000)
        })
        .catch(err => {
          console.error('Failed to copy code:', err)
        })
    }
  }
}

/**
 * Setup event delegation for copy code buttons
 * Should be called when component mounts
 */
export function setupMarkdownCopyFunction() {
  document.addEventListener('click', handleCopyCodeClick)
}

/**
 * Cleanup event delegation for copy code buttons
 * Should be called when component unmounts
 */
export function cleanupMarkdownCopyFunction() {
  document.removeEventListener('click', handleCopyCodeClick)
}

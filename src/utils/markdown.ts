import { marked } from 'marked'
import DOMPurify from 'dompurify'
import { logger } from './logger'

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
    // Convert markdown to HTML (sync form; no async rendering configured).
    // Cast is safe: marked.parse returns string in sync mode, but the type
    // union is `string | Promise<string>` so we narrow it explicitly.
    let html = marked.parse(markdown, { async: false }) as string

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
        /^(?:(?:(?:f|ht)tps?|mailto|tel|callto|cid|xmpp):|[^a-z]|[a-z+.-]+(?:[^a-z+.-:]|$))/i
    })

    return sanitized
  } catch (error) {
    logger.error('Error rendering markdown:', error)
    // Fallback to escaped plain text
    return escapeHtml(markdown)
  }
}

function addCustomClasses(html: string): string {
  // Parse the HTML into a transient container and mutate the DOM directly
  // instead of running chains of regexes (which break on attribute order or
  // existing class attributes).
  const container = document.createElement('div')
  container.innerHTML = html

  // Headings: add level-specific class
  for (let level = 1; level <= 6; level++) {
    container.querySelectorAll(`h${level}`).forEach(el => {
      el.classList.add('markdown-heading', `markdown-h${level}`)
    })
  }

  // Tables: wrap in a scroll container and add a class for styling
  container.querySelectorAll('table').forEach(table => {
    table.classList.add('markdown-table')
    const wrapper = document.createElement('div')
    wrapper.className = 'table-wrapper'
    table.parentNode?.insertBefore(wrapper, table)
    wrapper.appendChild(table)
  })

  // Blockquotes and lists
  container.querySelectorAll('blockquote').forEach(el => el.classList.add('markdown-blockquote'))
  container.querySelectorAll('ul, ol').forEach(el => el.classList.add('markdown-list'))

  // External links: ensure rel includes noopener/noreferrer when target=_blank
  container.querySelectorAll('a[target="_blank"]').forEach(a => {
    const existing = a.getAttribute('rel')
    if (!existing) {
      a.setAttribute('rel', 'noopener noreferrer')
    } else {
      const parts = new Set(existing.split(/\s+/).filter(Boolean))
      parts.add('noopener')
      parts.add('noreferrer')
      a.setAttribute('rel', Array.from(parts).join(' '))
    }
  })

  // Code blocks: wrap with a copy button
  container.querySelectorAll('pre > code').forEach(code => {
    const pre = code.parentElement
    if (!pre || pre.classList.contains('code-block')) return
    pre.classList.add('code-block')
    const button = document.createElement('button')
    button.className = 'copy-code-btn'
    button.setAttribute('data-copy-code', '')
    button.textContent = 'Copy'
    pre.appendChild(button)
  })

  return container.innerHTML
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
          target.textContent = 'Copied'
          setTimeout(() => {
            target.textContent = originalText
          }, 1000)
        })
        .catch(err => {
          logger.error('Failed to copy code:', err)
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

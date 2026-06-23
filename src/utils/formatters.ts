/**
 * Format a model name for display in the UI.
 *
 * Gemini-specific: assumes the input starts with the `gemini-` prefix and
 * is split into segments by `-`. Non-Gemini model names will still render
 * (the prefix is replaced and the rest is title-cased) but are not the
 * intended use case.
 *
 * @param model The raw model identifier, e.g. `"gemini-3-flash-preview"`.
 * @returns A human-readable display string, e.g. `"Gemini 3 Flash Preview"`.
 */
export function formatModelName(model: string): string {
  return model
    .replace('gemini-', 'Gemini ')
    .replace('-', ' ')
    .split(' ')
    .map(word => word.charAt(0).toUpperCase() + word.slice(1))
    .join(' ')
}

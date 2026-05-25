export function formatModelName(model: string): string {
  return model
    .replace('gemini-', 'Gemini ')
    .replace('-', ' ')
    .split(' ')
    .map(word => word.charAt(0).toUpperCase() + word.slice(1))
    .join(' ')
}

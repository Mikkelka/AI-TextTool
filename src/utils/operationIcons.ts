import type { Component } from 'vue'
import {
  AlignLeft,
  Briefcase,
  Languages,
  List,
  MessageCircle,
  Minimize2,
  Pen,
  Pencil,
  Smile,
  Sparkles,
  Wand2
} from '@lucide/vue'

const ICON_MAP: Record<string, Component> = {
  pencil: Pencil,
  rewrite: Pen,
  translate: Languages,
  concise: Minimize2,
  'smiley-face': Smile,
  briefcase: Briefcase,
  list: List,
  summary: AlignLeft,
  chat: MessageCircle,
  wand: Wand2
}

export const getOperationIcon = (iconName: string | undefined): Component => {
  if (iconName && ICON_MAP[iconName]) {
    return ICON_MAP[iconName]
  }
  return Sparkles
}

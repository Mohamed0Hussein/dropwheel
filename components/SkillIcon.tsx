'use client';

import {
  Braces,
  FileType,
  FileType2,
  FileCode,
  Atom,
  Component,
  Wind,
  Palette,
  Framer,
  Boxes,
  Hexagon,
  Server,
  Network,
  Route,
  Waypoints,
  Leaf,
  Database,
  Layers,
  GitBranch,
  Workflow,
  Container,
  Triangle,
  Zap,
  TestTube2,
  Figma,
  Send,
  Code2,
  type LucideIcon,
} from 'lucide-react';

/**
 * Maps a technology label to a representative (not brand-traced) icon.
 * Keeps everything on the already-installed lucide-react set rather than
 * hand-tracing third-party logos — same visual intent, no trademark risk.
 */
const ICON_MAP: Record<string, LucideIcon> = {
  'JavaScript (ES6+)': Braces,
  TypeScript: FileType2,
  'React 18': Atom,
  React: Atom,
  'Next.js 15': Component,
  'Next.js 14/15': Component,
  'Next.js': Component,
  'Next.js API Routes': Route,
  'Redux Toolkit': Boxes,
  'Tailwind CSS v4': Wind,
  'Tailwind CSS': Wind,
  'Material UI': Palette,
  'Framer Motion': Framer,
  'Radix UI': Component,
  'CSS3 / Sass': FileType,
  'CSS3/Sass': FileType,
  HTML5: FileCode,
  'Node.js': Hexagon,
  'Express.js': Server,
  'REST APIs': Network,
  GraphQL: Waypoints,
  'GraphQL Basics': Waypoints,
  MongoDB: Leaf,
  Mongoose: Leaf,
  PostgreSQL: Database,
  'Prisma ORM': Layers,
  Git: GitBranch,
  'GitHub Actions': Workflow,
  Docker: Container,
  Vercel: Triangle,
  Vite: Zap,
  Jest: TestTube2,
  Figma: Figma,
  Postman: Send,
};

export function getSkillIcon(label: string): LucideIcon {
  return ICON_MAP[label] ?? Code2;
}

export default function SkillIcon({ label, className }: { label: string; className?: string }) {
  // Not a dynamically-created component — ICON_MAP only ever returns stable,
  // module-level lucide-react components, so identity is stable across renders.
  /* eslint-disable react-hooks/static-components */
  const Icon = getSkillIcon(label);
  return <Icon className={className} aria-hidden="true" />;
  /* eslint-enable react-hooks/static-components */
}

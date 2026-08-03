'use client';

import { AnimatePresence, motion, useMotionTemplate, useMotionValue, useSpring, useTransform } from 'framer-motion';
import { ArrowUpRight, Github, ExternalLink, Star } from 'lucide-react';
import Link from 'next/link';
import { MouseEvent, useState } from 'react';
import ProjectFilterTabs from './ProjectFilterTabs';
import ProjectDetailModal from './ProjectDetailModal';

export type ProjectCategory = 'Frontend' | 'Full-Stack';

export type Project = {
  title: string;
  subtitle: string;
  description: string;
  category: ProjectCategory;
  tech: string[];
  live?: string;
  github?: string;
  featured?: boolean;
  gradient: string;
  pattern: string;
  accentColor: string;
  architecture: string;
  challenges: string;
  features: string[];
  lessonsLearned: string;
};

export const projects: Project[] = [
  {
    title: 'Second Mind',
    subtitle: 'Productivity Tool',
    description:
      'Intelligent note-taking with force-directed graph visualization. Connect ideas visually and rediscover insights through interactive knowledge maps.',
    category: 'Frontend',
    tech: ['React', 'Next.js', 'TypeScript', 'Canvas API'],
    live: 'https://second-brain-ebon.vercel.app/',
    github: 'https://github.com/Mohamed0Hussein/second-brain',
    featured: true,
    gradient: 'linear-gradient(135deg, #7c3aed 0%, #3b82f6 50%, #22d3ee 100%)',
    pattern: 'radial-gradient(circle at 20% 80%, rgba(124,58,237,0.5) 0%, transparent 50%), radial-gradient(circle at 80% 20%, rgba(34,211,238,0.4) 0%, transparent 50%), radial-gradient(circle at 50% 50%, rgba(59,130,246,0.2) 0%, transparent 60%)',
    accentColor: '#7c3aed',
    architecture:
      'Client-side React/Next.js app rendering notes as nodes on an HTML5 Canvas, with a force-directed graph layout recalculated on every edge change and persisted to local storage.',
    challenges:
      'The hardest part was keeping the force simulation smooth at 60fps once a graph grew past a couple hundred nodes — required throttling layout recalculation and only re-rendering the canvas region that changed.',
    features: [
      'Force-directed graph visualization of linked notes',
      'Freeform canvas panning and zooming',
      'Bi-directional linking between notes with live preview',
      'Local-first persistence, no backend required',
    ],
    lessonsLearned:
      'Canvas-based interaction state (drag, zoom, selection) is much easier to reason about once it is modeled as a small state machine instead of ad-hoc mouse-event flags.',
  },
  {
    title: 'Talabat ElBeit',
    subtitle: 'Full-Stack Application',
    description:
      'End-to-end food delivery platform with real-time cart management, order tracking, and a streamlined checkout experience.',
    category: 'Full-Stack',
    tech: ['React', 'Node.js', 'MongoDB', 'Express'],
    github: 'https://github.com/Mohamed0Hussein/Talabat_ElBeit',
    featured: true,
    gradient: 'linear-gradient(135deg, #3b82f6 0%, #0ea5e9 50%, #22d3ee 100%)',
    pattern: 'radial-gradient(circle at 30% 70%, rgba(59,130,246,0.5) 0%, transparent 50%), radial-gradient(circle at 70% 30%, rgba(34,211,238,0.4) 0%, transparent 50%)',
    accentColor: '#3b82f6',
    architecture:
      'React SPA talking to an Express/Node REST API backed by MongoDB, with cart state synced to the server on every mutation so checkout survives a page reload.',
    challenges:
      'Keeping cart totals, delivery fees, and item availability consistent across client and server needed a single source of truth on the backend, with the client treated as a thin, always-refetching view.',
    features: [
      'Real-time cart with quantity and availability sync',
      'Order tracking from checkout to delivery',
      'Streamlined, multi-step checkout flow',
      'RESTful API with MongoDB persistence',
    ],
    lessonsLearned:
      'Full-stack ownership end-to-end (schema, API, UI) made it obvious how much easier debugging is when the client never has to guess server state.',
  },
  {
    title: 'Stories App',
    subtitle: 'Social Media',
    description:
      'Instagram-inspired ephemeral stories platform with smooth transitions, story progress bars, and swipe navigation.',
    category: 'Frontend',
    tech: ['React', 'Next.js', 'Framer Motion', 'CSS'],
    live: 'https://stories-app-two.vercel.app',
    featured: true,
    gradient: 'linear-gradient(135deg, #06b6d4 0%, #3b82f6 50%, #7c3aed 100%)',
    pattern: 'radial-gradient(circle at 20% 20%, rgba(6,182,212,0.5) 0%, transparent 50%), radial-gradient(circle at 80% 80%, rgba(124,58,237,0.4) 0%, transparent 50%)',
    accentColor: '#06b6d4',
    architecture:
      'Next.js app with a story-viewer component driven by a timer-based progress state machine, swapping active story/segment on a `requestAnimationFrame` progress loop.',
    challenges:
      'Getting swipe-to-advance and tap-to-pause to feel native (not laggy) meant moving progress-bar animation off React state and onto Framer Motion values driven directly by the frame loop.',
    features: [
      'Auto-advancing story progress bars per segment',
      'Swipe and tap navigation between stories',
      'Pause-on-hold interaction',
      'Smooth cross-fade transitions between stories',
    ],
    lessonsLearned:
      'Animation-heavy UI is far smoother when the animated value lives outside React state entirely — Framer Motion values plus refs beat setState-per-frame every time.',
  },
  {
    title: 'Space Tourism',
    subtitle: 'Interactive Website',
    description:
      'Immersive multi-page space tourism site with animated page transitions, responsive layouts, and rich visual storytelling.',
    category: 'Frontend',
    tech: ['React', 'CSS Modules', 'Framer Motion'],
    github: 'https://github.com/Mohamed0Hussein/Space-tourism',
    gradient: 'linear-gradient(135deg, #14b8a6 0%, #0ea5e9 50%, #3b82f6 100%)',
    pattern: 'radial-gradient(circle at 50% 50%, rgba(20,184,166,0.5) 0%, transparent 60%), radial-gradient(circle at 80% 20%, rgba(59,130,246,0.3) 0%, transparent 50%)',
    accentColor: '#14b8a6',
    architecture:
      'Multi-route React app with CSS Modules for per-page scoped styling and Framer Motion `AnimatePresence` handling cross-route enter/exit transitions.',
    challenges:
      'Matching the original Frontend Mentor design at every one of its three breakpoints exactly (desktop/tablet/mobile nav all differ structurally) took more layout iteration than the animation work itself.',
    features: [
      'Animated route transitions between Home/Destination/Crew/Technology',
      'Responsive layouts with distinct mobile/tablet/desktop navigation',
      'Background imagery swapped per breakpoint',
      'Tabbed content switchers for destinations and crew',
    ],
    lessonsLearned:
      'Pixel-accurate responsive design is a discipline of its own, independent of animation polish — get the breakpoints right first.',
  },
  {
    title: 'Login Form',
    subtitle: 'UI Component',
    description:
      'Modern authentication form with real-time validation, smooth micro-animations, and accessible design patterns.',
    category: 'Frontend',
    tech: ['React', 'TypeScript', 'Framer Motion'],
    live: 'https://login-form-five-blue.vercel.app',
    gradient: 'linear-gradient(135deg, #0d9488 0%, #22d3ee 100%)',
    pattern: 'radial-gradient(circle at 30% 50%, rgba(13,148,136,0.5) 0%, transparent 55%), radial-gradient(circle at 70% 70%, rgba(34,211,238,0.3) 0%, transparent 50%)',
    accentColor: '#0d9488',
    architecture:
      'Small, self-contained React + TypeScript form component with schema-driven validation rules and Framer Motion for field-level error/success micro-animations.',
    challenges:
      'Balancing "validate as you type" against not being annoying required debouncing validation until the user pauses or blurs the field, rather than validating on every keystroke.',
    features: [
      'Real-time field validation with inline error states',
      'Accessible labels, error announcements, and focus management',
      'Animated success/error micro-interactions',
      'Fully typed form state with TypeScript',
    ],
    lessonsLearned:
      'Accessible form validation and "delightful" animation are not in tension once validation timing is decoupled from animation timing.',
  },
  {
    title: 'Age Calculator',
    subtitle: 'Web App',
    description:
      'Precision date calculator with animated number transitions, input validation, and a clean, minimal interface.',
    category: 'Frontend',
    tech: ['React', 'TypeScript', 'CSS'],
    live: 'https://age-calculator-sand-rho.vercel.app',
    gradient: 'linear-gradient(135deg, #6366f1 0%, #3b82f6 50%, #22d3ee 100%)',
    pattern: 'radial-gradient(circle at 70% 30%, rgba(99,102,241,0.5) 0%, transparent 55%), radial-gradient(circle at 30% 70%, rgba(34,211,238,0.3) 0%, transparent 50%)',
    accentColor: '#6366f1',
    architecture:
      'Single-page React + TypeScript app computing calendar-accurate day/month/year differences (handling leap years and variable month lengths) with animated digit transitions on result.',
    challenges:
      'Calendar math has more edge cases than it looks like — validating a real, in-range date and computing an accurate diff across leap years took more care than the UI.',
    features: [
      'Calendar-accurate age calculation (leap years, variable month lengths)',
      'Inline input validation with specific error messages',
      'Animated digit roll-up on result',
      'Minimal, focused single-purpose interface',
    ],
    lessonsLearned:
      'Small "just a form" apps often hide the most finicky logic — investing in edge-case tests early paid off more here than in some larger projects.',
  },
];

/* ── Individual card ── */
function ProjectCard({ project, index, onOpen }: { project: Project; index: number; onOpen: () => void }) {
  // ── Spotlight: absolute-pixel coords fed into the CSS radial-gradient
  const spotX = useMotionValue(0);
  const spotY = useMotionValue(0);

  // ── Tilt: normalised –0.5 → +0.5
  const tiltX = useMotionValue(0);
  const tiltY = useMotionValue(0);

  const springCfg = { damping: 28, stiffness: 200 };
  const rotateX = useSpring(useTransform(tiltY, [-0.5, 0.5], [7, -7]), springCfg);
  const rotateY = useSpring(useTransform(tiltX, [-0.5, 0.5], [-7, 7]), springCfg);

  const spotlightBg = useMotionTemplate`radial-gradient(600px circle at ${spotX}px ${spotY}px, ${project.accentColor}18, transparent 80%)`;

  function handleMouseMove({ currentTarget, clientX, clientY }: MouseEvent) {
    const { left, top, width, height } = currentTarget.getBoundingClientRect();
    // Spotlight uses raw pixel coords
    spotX.set(clientX - left);
    spotY.set(clientY - top);
    // Tilt uses normalised coords (–0.5 → +0.5)
    tiltX.set((clientX - left) / width  - 0.5);
    tiltY.set((clientY - top)  / height - 0.5);
  }

  function handleMouseLeave() {
    // Spring back to flat
    tiltX.set(0);
    tiltY.set(0);
  }

  return (
    <motion.article
      layout
      initial={{ opacity: 0, y: 40, scale: 0.95 }}
      whileInView={{ opacity: 1, y: 0, scale: 1 }}
      viewport={{ once: true, margin: '-40px' }}
      exit={{ opacity: 0, y: -16, scale: 0.96 }}
      transition={{ duration: 0.5, delay: index * 0.06, ease: [0.25, 1, 0.5, 1] }}
      className="card group relative flex flex-col gradient-border-card cursor-pointer text-left"
      style={{
        rotateX,
        rotateY,
        transformPerspective: 1200,
      }}
      onMouseMove={handleMouseMove}
      onMouseLeave={handleMouseLeave}
      onClick={onOpen}
      role="button"
      tabIndex={0}
      aria-label={`View details for ${project.title}`}
      onKeyDown={(e) => {
        if (e.key === 'Enter' || e.key === ' ') {
          e.preventDefault();
          onOpen();
        }
      }}
    >
      {/* Mouse spotlight */}
      <motion.div
        className="pointer-events-none absolute inset-0 rounded-[20px] opacity-0 transition duration-500 group-hover:opacity-100 z-0"
        style={{ background: spotlightBg }}
      />

      {/* Preview header */}
      <div
        className="relative overflow-hidden flex-shrink-0"
        style={{ height: '200px', background: project.pattern, backgroundColor: 'rgba(0,0,0,0.04)' }}
        aria-hidden
      >
        {/* Geometric decorations inside preview */}
        <div className="absolute inset-0 flex items-center justify-center">
          <motion.div
            className="w-20 h-20 rounded-3xl rotate-12 opacity-20 border-2 border-white"
            style={{ background: 'rgba(255,255,255,0.12)' }}
            animate={{ rotate: [12, 20, 12], scale: [1, 1.08, 1] }}
            transition={{ duration: 8, repeat: Infinity, ease: 'easeInOut' }}
          />
          <motion.div
            className="absolute w-32 h-32 rounded-[40%] -rotate-6 opacity-[0.08] border border-white"
            style={{ background: 'rgba(255,255,255,0.06)' }}
            animate={{ rotate: [-6, -14, -6], scale: [1, 1.1, 1] }}
            transition={{ duration: 12, repeat: Infinity, ease: 'easeInOut', delay: 2 }}
          />
          <div
            className="absolute w-10 h-10 rounded-full opacity-30"
            style={{ background: 'rgba(255,255,255,0.3)', top: '20%', left: '15%', filter: 'blur(6px)' }}
          />
        </div>

        {/* Project title overlay in preview */}
        <div className="absolute inset-0 flex items-end p-5 z-10">
          <div
            className="text-white/20 font-black font-display text-5xl leading-none tracking-tighter select-none"
            style={{ letterSpacing: '-0.05em' }}
          >
            {project.title.charAt(0)}
          </div>
        </div>

        {/* Top gradient accent bar */}
        <div
          className="absolute top-0 left-0 right-0 h-[3px] rounded-t-[20px] transition-all duration-500 origin-left group-hover:h-[4px]"
          style={{ background: project.gradient }}
        />

        {/* Badges */}
        <div className="absolute top-4 right-4 flex gap-2 z-10">
          {project.featured && (
            <span
              className="text-[10px] font-bold px-2.5 py-1 rounded-full border backdrop-blur-md flex items-center gap-1"
              style={{
                background: 'rgba(255,255,255,0.14)',
                borderColor: 'rgba(255,255,255,0.3)',
                color: '#ffffff',
              }}
            >
              <Star className="w-2.5 h-2.5 fill-white" />
              Featured
            </span>
          )}
        </div>
      </div>

      {/* Card body */}
      <div className="flex flex-col flex-1 p-7 pt-6 relative z-10">
        {/* Category label */}
        <span
          className="text-[0.65rem] font-bold uppercase tracking-[0.24em] mb-3"
          style={{ color: project.accentColor, opacity: 0.8 }}
        >
          {project.subtitle}
        </span>

        {/* Title */}
        <h3
          className="text-xl font-display font-bold mb-3 transition-colors duration-300 group-hover:text-accent tracking-tight"
          style={{ color: 'var(--text-primary)', letterSpacing: '-0.02em' }}
        >
          {project.title}
        </h3>

        {/* Description */}
        <p
          className="text-[0.9rem] leading-relaxed mb-6 flex-1"
          style={{ color: 'var(--text-secondary)', fontWeight: 400 }}
        >
          {project.description}
        </p>

        {/* Tech tags */}
        <div className="flex flex-wrap gap-2 mb-6">
          {project.tech.map((t) => (
            <span
              key={t}
              className="text-[0.7rem] font-semibold px-2.5 py-1 rounded-lg border transition-all duration-300 group-hover:border-accent/40"
              style={{
                background: 'var(--surface-elevated)',
                borderColor: 'var(--border-strong)',
                color: 'var(--text-secondary)',
                fontFamily: 'JetBrains Mono, var(--font-mono), monospace',
              }}
            >
              {t}
            </span>
          ))}
        </div>

        {/* Links */}
        <div
          className="flex items-center gap-3 pt-5 mt-auto"
          style={{ borderTop: '1px solid var(--border)' }}
        >
          {project.live && (
            <motion.div whileHover={{ scale: 1.04 }} whileTap={{ scale: 0.96 }} onClick={(e) => e.stopPropagation()}>
              <Link
                href={project.live}
                target="_blank"
                rel="noopener noreferrer"
                className="btn-primary py-2! px-5! text-sm! rounded-xl gap-1.5"
                aria-label={`Live demo of ${project.title}`}
              >
                <ExternalLink className="w-3.5 h-3.5" />
                Live Demo
              </Link>
            </motion.div>
          )}
          {project.github && (
            <motion.div whileHover={{ scale: 1.04 }} whileTap={{ scale: 0.96 }} onClick={(e) => e.stopPropagation()}>
              <Link
                href={project.github}
                target="_blank"
                rel="noopener noreferrer"
                className="btn-outline py-2! px-5! text-sm! rounded-xl gap-1.5"
                aria-label={`Source code of ${project.title}`}
              >
                <Github className="w-3.5 h-3.5" />
                Source
              </Link>
            </motion.div>
          )}

          {/* Arrow indicator */}
          <motion.div
            className="ml-auto"
            initial={{ opacity: 0, x: -4 }}
            whileHover={{ opacity: 1, x: 0 }}
          >
            <ArrowUpRight
              className="w-4 h-4 opacity-0 group-hover:opacity-100 transition-all duration-300 -translate-x-1 group-hover:translate-x-0"
              style={{ color: 'var(--accent)' }}
            />
          </motion.div>
        </div>
      </div>
    </motion.article>
  );
}

const FILTERS = ['All', 'Featured', 'Full-Stack', 'Frontend'] as const;

export default function Projects() {
  const [activeFilter, setActiveFilter] = useState<(typeof FILTERS)[number]>('All');
  const [selectedProject, setSelectedProject] = useState<Project | null>(null);

  const filteredProjects = projects.filter((project) => {
    if (activeFilter === 'All') return true;
    if (activeFilter === 'Featured') return Boolean(project.featured);
    return project.category === activeFilter;
  });

  return (
    <section id="projects" className="py-28 md:py-36 px-6 md:px-8 relative z-10 overflow-hidden">

      {/* Background accent */}
      <div
        className="absolute top-0 left-1/2 -translate-x-1/2 w-[800px] h-[400px] rounded-full pointer-events-none -z-10"
        style={{
          background: 'radial-gradient(ellipse at center, var(--mesh-1) 0%, transparent 70%)',
          filter: 'blur(80px)',
          opacity: 0.6,
        }}
      />

      <div className="max-w-6xl mx-auto">

        {/* Header */}
        <motion.div
          initial={{ opacity: 0, y: 28 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.9, ease: [0.25, 1, 0.5, 1] }}
          className="mb-10"
        >
          <span className="section-label">Portfolio</span>
          <h2 className="section-title text-5xl md:text-6xl lg:text-7xl mb-5 tracking-tighter">
            Selected work<span style={{ color: 'var(--accent)' }}>.</span>
          </h2>
          <p className="section-subtitle font-light md:text-xl">
            Projects that showcase my ability to ship polished, production-grade applications
            that blend form and function perfectly.
          </p>
        </motion.div>

        {/* Filter tabs */}
        <div className="mb-10">
          <ProjectFilterTabs filters={[...FILTERS]} active={activeFilter} onChange={(f) => setActiveFilter(f as typeof activeFilter)} />
        </div>

        {/* Grid */}
        <motion.div layout className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 md:gap-7">
          <AnimatePresence mode="popLayout">
            {filteredProjects.map((project, index) => (
              <ProjectCard
                key={project.title}
                project={project}
                index={index}
                onOpen={() => setSelectedProject(project)}
              />
            ))}
          </AnimatePresence>
        </motion.div>

        {filteredProjects.length === 0 && (
          <p className="text-center py-16" style={{ color: 'var(--text-tertiary)' }}>
            No projects in this category yet.
          </p>
        )}

        {/* View all CTA */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ delay: 0.4, duration: 0.7 }}
          className="mt-16 text-center"
        >
          <motion.div whileHover={{ scale: 1.03 }} whileTap={{ scale: 0.97 }} className="inline-block">
            <Link
              href="https://github.com/Mohamed0Hussein?tab=repositories"
              target="_blank"
              rel="noopener noreferrer"
              className="btn-outline px-8 py-4 rounded-full text-base"
              aria-label="Explore all GitHub repositories"
            >
              <Github className="w-4 h-4" />
              Explore all repositories
              <ArrowUpRight className="w-4 h-4" />
            </Link>
          </motion.div>
        </motion.div>
      </div>

      <AnimatePresence>
        {selectedProject && (
          <ProjectDetailModal project={selectedProject} onClose={() => setSelectedProject(null)} />
        )}
      </AnimatePresence>
    </section>
  );
}

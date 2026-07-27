'use client';

import { motion, useMotionTemplate, useMotionValue } from 'framer-motion';
import { ArrowUpRight, Github, ExternalLink } from 'lucide-react';
import Link from 'next/link';
import { MouseEvent } from 'react';

type Project = {
  title: string;
  subtitle: string;
  description: string;
  tech: string[];
  live?: string;
  github?: string;
  featured?: boolean;
  gradient: string;
  pattern: string; // CSS background pattern
};

const projects: Project[] = [
  {
    title: 'Second Mind',
    subtitle: 'Productivity Tool',
    description:
      'Intelligent note-taking with force-directed graph visualization. Connect ideas visually and rediscover insights through interactive knowledge maps.',
    tech: ['React', 'Next.js', 'TypeScript', 'Canvas API'],
    live: 'https://second-brain-ebon.vercel.app/',
    github: 'https://github.com/Mohamed0Hussein/second-brain',
    featured: true,
    gradient: 'linear-gradient(135deg, #6366f1 0%, #8b5cf6 50%, #a855f7 100%)',
    pattern: 'radial-gradient(circle at 20% 80%, rgba(99,102,241,0.4) 0%, transparent 50%), radial-gradient(circle at 80% 20%, rgba(168,85,247,0.3) 0%, transparent 50%)',
  },
  {
    title: 'Talabat ElBeit',
    subtitle: 'Full-Stack Application',
    description:
      'End-to-end food delivery platform with real-time cart management, order tracking, and a streamlined checkout experience.',
    tech: ['React', 'Node.js', 'MongoDB', 'Express'],
    github: 'https://github.com/Mohamed0Hussein/Talabat_ElBeit',
    featured: true,
    gradient: 'linear-gradient(135deg, #f59e0b 0%, #ef4444 50%, #ec4899 100%)',
    pattern: 'radial-gradient(circle at 30% 70%, rgba(245,158,11,0.4) 0%, transparent 50%), radial-gradient(circle at 70% 30%, rgba(236,72,153,0.3) 0%, transparent 50%)',
  },
  {
    title: 'Stories App',
    subtitle: 'Social Media',
    description:
      'Instagram-inspired ephemeral stories platform with smooth transitions, story progress bars, and swipe navigation.',
    tech: ['React', 'Next.js', 'Framer Motion', 'CSS'],
    live: 'https://stories-app-two.vercel.app',
    featured: true,
    gradient: 'linear-gradient(135deg, #06b6d4 0%, #3b82f6 50%, #6366f1 100%)',
    pattern: 'radial-gradient(circle at 20% 20%, rgba(6,182,212,0.4) 0%, transparent 50%), radial-gradient(circle at 80% 80%, rgba(99,102,241,0.3) 0%, transparent 50%)',
  },
  {
    title: 'Space Tourism',
    subtitle: 'Interactive Website',
    description:
      'Immersive multi-page space tourism site with animated page transitions, responsive layouts, and rich visual storytelling.',
    tech: ['React', 'CSS Modules', 'Framer Motion'],
    github: 'https://github.com/Mohamed0Hussein/Space-tourism',
    gradient: 'linear-gradient(135deg, #0ea5e9 0%, #6366f1 50%, #8b5cf6 100%)',
    pattern: 'radial-gradient(circle at 50% 50%, rgba(14,165,233,0.4) 0%, transparent 60%)',
  },
  {
    title: 'Login Form',
    subtitle: 'UI Component',
    description:
      'Modern authentication form with real-time validation, smooth micro-animations, and accessible design patterns.',
    tech: ['React', 'TypeScript', 'Framer Motion'],
    live: 'https://login-form-five-blue.vercel.app',
    gradient: 'linear-gradient(135deg, #10b981 0%, #059669 50%, #047857 100%)',
    pattern: 'radial-gradient(circle at 30% 50%, rgba(16,185,129,0.4) 0%, transparent 55%)',
  },
  {
    title: 'Age Calculator',
    subtitle: 'Web App',
    description:
      'Precision date calculator with animated number transitions, input validation, and a clean, minimal interface.',
    tech: ['React', 'TypeScript', 'CSS'],
    live: 'https://age-calculator-sand-rho.vercel.app',
    gradient: 'linear-gradient(135deg, #f43f5e 0%, #e11d48 50%, #be123c 100%)',
    pattern: 'radial-gradient(circle at 70% 30%, rgba(244,63,94,0.4) 0%, transparent 55%)',
  },
];

/* ── Individual card ── */
function ProjectCard({ project, index }: { project: Project; index: number }) {
  const mouseX = useMotionValue(0);
  const mouseY = useMotionValue(0);

  function handleMouseMove({ currentTarget, clientX, clientY }: MouseEvent) {
    const { left, top } = currentTarget.getBoundingClientRect();
    mouseX.set(clientX - left);
    mouseY.set(clientY - top);
  }

  const spotlightBg = useMotionTemplate`radial-gradient(550px circle at ${mouseX}px ${mouseY}px, var(--accent-glow), transparent 80%)`;

  return (
    <motion.article
      initial={{ opacity: 0, y: 32, scale: 0.96 }}
      whileInView={{ opacity: 1, y: 0, scale: 1 }}
      viewport={{ once: true, margin: '-40px' }}
      transition={{ duration: 0.65, delay: index * 0.08, ease: [0.25, 1, 0.5, 1] }}
      className="card group relative flex flex-col gradient-border-card"
      onMouseMove={handleMouseMove}
    >
      {/* Mouse spotlight */}
      <motion.div
        className="pointer-events-none absolute -inset-px rounded-[20px] opacity-0 transition duration-500 group-hover:opacity-100 z-0"
        style={{ background: spotlightBg }}
      />

      {/* Preview header */}
      <div
        className="project-preview rounded-t-[20px]"
        style={{ background: project.pattern, backgroundColor: 'rgba(0,0,0,0.05)' }}
        aria-hidden
      >
        {/* Geometric decoration */}
        <div className="absolute inset-0 flex items-center justify-center pointer-events-none">
          <div
            className="w-16 h-16 rounded-2xl rotate-12 opacity-20 border-2 border-white"
            style={{ background: 'rgba(255,255,255,0.15)' }}
          />
          <div
            className="absolute w-24 h-24 rounded-3xl -rotate-6 opacity-10 border border-white"
            style={{ background: 'rgba(255,255,255,0.1)' }}
          />
        </div>

        {/* Gradient accent bar at top */}
        <div
          className="absolute top-0 left-0 right-0 h-1 rounded-t-[20px] transition-transform duration-500 origin-left"
          style={{ background: project.gradient }}
        />

        {/* Featured / subtitle chip */}
        <div className="absolute top-4 right-4 flex gap-2">
          {project.featured && (
            <span
              className="text-[10px] font-bold px-2.5 py-1 rounded-full border backdrop-blur-md"
              style={{
                background: 'rgba(255,255,255,0.12)',
                borderColor: 'rgba(255,255,255,0.25)',
                color: '#ffffff',
              }}
            >
              Featured
            </span>
          )}
        </div>
      </div>

      {/* Card body */}
      <div className="flex flex-col flex-1 p-7 pt-6 relative z-10">
        {/* Category label */}
        <span
          className="text-[0.65rem] font-bold uppercase tracking-[0.22em] mb-3"
          style={{ color: 'var(--text-tertiary)' }}
        >
          {project.subtitle}
        </span>

        {/* Title */}
        <h3
          className="text-xl font-display font-bold mb-3 transition-colors duration-300 group-hover:text-accent tracking-tight"
          style={{ color: 'var(--text-primary)' }}
        >
          {project.title}
        </h3>

        {/* Description */}
        <p
          className="text-[0.9rem] leading-relaxed mb-6 font-medium flex-1"
          style={{ color: 'var(--text-secondary)' }}
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
                fontFamily: 'var(--font-mono)',
              }}
            >
              {t}
            </span>
          ))}
        </div>

        {/* Links */}
        <div
          className="flex items-center gap-3 pt-5 mt-auto"
          style={{ borderTop: '1px solid var(--border-strong)' }}
        >
          {project.live && (
            <motion.div whileHover={{ scale: 1.04 }} whileTap={{ scale: 0.96 }}>
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
            <motion.div whileHover={{ scale: 1.04 }} whileTap={{ scale: 0.96 }}>
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
        </div>
      </div>
    </motion.article>
  );
}

export default function Projects() {
  return (
    <section id="projects" className="py-24 md:py-32 px-6 md:px-8 relative z-10">
      <div className="max-w-6xl mx-auto">

        {/* Header */}
        <motion.div
          initial={{ opacity: 0, y: 24 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.8, ease: [0.25, 1, 0.5, 1] }}
          className="mb-16"
        >
          <span className="section-label">Portfolio</span>
          <h2 className="section-title text-5xl md:text-6xl lg:text-7xl mb-5 tracking-tighter">
            Selected work<span className="text-accent">.</span>
          </h2>
          <p className="section-subtitle font-light md:text-xl">
            Projects that showcase my ability to ship polished, production-grade applications
            that blend form and function perfectly.
          </p>
        </motion.div>

        {/* Grid */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 md:gap-8">
          {projects.map((project, index) => (
            <ProjectCard key={project.title} project={project} index={index} />
          ))}
        </div>

        {/* View all CTA */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ delay: 0.4, duration: 0.6 }}
          className="mt-14 text-center"
        >
          <motion.div whileHover={{ scale: 1.03 }} whileTap={{ scale: 0.97 }} className="inline-block">
            <Link
              href="https://github.com/Mohamed0Hussein?tab=repositories"
              target="_blank"
              rel="noopener noreferrer"
              className="btn-outline px-8 py-4 rounded-full text-base"
              aria-label="Explore all GitHub repositories"
            >
              Explore all repositories
              <ArrowUpRight className="w-5 h-5" />
            </Link>
          </motion.div>
        </motion.div>
      </div>
    </section>
  );
}

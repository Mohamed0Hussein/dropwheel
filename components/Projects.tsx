'use client';

import { motion } from 'framer-motion';
import { ArrowUpRight, Github } from 'lucide-react';
import Link from 'next/link';

type ProjectType = {
  title: string;
  subtitle: string;
  description: string;
  tech: string[];
  live?: string;
  github?: string;
  featured?: boolean;
  gradient: string;
};

const projects: ProjectType[] = [
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
  },
  {
    title: 'Space Tourism',
    subtitle: 'Interactive Website',
    description:
      'Immersive multi-page space tourism site with animated page transitions, responsive layouts, and rich visual storytelling.',
    tech: ['React', 'CSS Modules', 'Framer Motion'],
    github: 'https://github.com/Mohamed0Hussein/Space-tourism',
    gradient: 'linear-gradient(135deg, #0ea5e9 0%, #6366f1 50%, #8b5cf6 100%)',
  },
  {
    title: 'Login Form',
    subtitle: 'UI Component',
    description:
      'Modern authentication form with real-time validation, smooth micro-animations, and accessible design patterns.',
    tech: ['React', 'TypeScript', 'Framer Motion'],
    live: 'https://login-form-five-blue.vercel.app',
    gradient: 'linear-gradient(135deg, #10b981 0%, #059669 50%, #047857 100%)',
  },
  {
    title: 'Age Calculator',
    subtitle: 'Web App',
    description:
      'Precision date calculator with animated number transitions, input validation, and a clean, minimal interface.',
    tech: ['React', 'TypeScript', 'CSS'],
    live: 'https://age-calculator-sand-rho.vercel.app',
    gradient: 'linear-gradient(135deg, #f43f5e 0%, #e11d48 50%, #be123c 100%)',
  },
];

function ProjectCard({ project, index }: { project: ProjectType; index: number }) {
  return (
    <motion.div
      initial={{ opacity: 0, y: 24 }}
      whileInView={{ opacity: 1, y: 0 }}
      viewport={{ once: true, margin: '-50px' }}
      transition={{ duration: 0.5, delay: index * 0.08 }}
      className="card group"
    >
      {/* Gradient Header Bar */}
      <div className="h-1.5 w-full" style={{ background: project.gradient }} />

      <div className="p-6 md:p-7">
        {/* Category + Featured Tag */}
        <div className="flex items-center justify-between mb-4">
          <span
            className="text-xs font-semibold uppercase tracking-wider"
            style={{ color: 'var(--text-tertiary)' }}
          >
            {project.subtitle}
          </span>
          {project.featured && (
            <span
              className="text-xs font-medium px-2.5 py-1 rounded-md"
              style={{
                background: 'var(--badge-bg)',
                color: 'var(--badge-text)',
                border: '1px solid var(--badge-border)',
              }}
            >
              Featured
            </span>
          )}
        </div>

        {/* Title */}
        <h3
          className="text-xl font-bold mb-3 transition-colors duration-300"
          style={{ color: 'var(--text-primary)', letterSpacing: '-0.01em' }}
        >
          {project.title}
        </h3>

        {/* Description */}
        <p
          className="text-sm leading-relaxed mb-5"
          style={{ color: 'var(--text-secondary)' }}
        >
          {project.description}
        </p>

        {/* Tech Stack */}
        <div className="flex flex-wrap gap-2 mb-6">
          {project.tech.map((tech) => (
            <span
              key={tech}
              className="text-xs font-medium px-2.5 py-1 rounded-md"
              style={{
                background: 'var(--code-bg)',
                color: 'var(--text-secondary)',
                fontFamily: 'var(--font-mono)',
              }}
            >
              {tech}
            </span>
          ))}
        </div>

        {/* Action Links */}
        <div
          className="flex items-center gap-3 pt-4"
          style={{ borderTop: '1px solid var(--border)' }}
        >
          {project.live && (
            <Link
              href={project.live}
              target="_blank"
              rel="noopener noreferrer"
              className="btn-ghost group/link"
            >
              Live Demo
              <ArrowUpRight className="w-3.5 h-3.5 transition-transform group-hover/link:translate-x-0.5 group-hover/link:-translate-y-0.5" />
            </Link>
          )}
          {project.github && (
            <Link
              href={project.github}
              target="_blank"
              rel="noopener noreferrer"
              className="btn-ghost group/link"
            >
              <Github className="w-3.5 h-3.5" />
              Source
            </Link>
          )}
        </div>
      </div>
    </motion.div>
  );
}

export default function Projects() {
  return (
    <section id="projects" className="py-24 md:py-32 px-6 md:px-8">
      <div className="max-w-6xl mx-auto">
        <motion.div
          initial={{ opacity: 0, y: 24 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.6 }}
          className="mb-12"
        >
          <span className="section-label">Portfolio</span>
          <h2 className="section-title text-4xl md:text-5xl lg:text-6xl mb-4">
            Selected work
          </h2>
          <p className="section-subtitle">
            Projects that showcase my ability to ship polished, production-grade applications.
          </p>
        </motion.div>

        {/* Projects Grid */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-5">
          {projects.map((project, index) => (
            <ProjectCard key={project.title} project={project} index={index} />
          ))}
        </div>

        {/* View All */}
        <motion.div
          initial={{ opacity: 0 }}
          whileInView={{ opacity: 1 }}
          viewport={{ once: true }}
          transition={{ delay: 0.4 }}
          className="mt-12 text-center"
        >
          <Link
            href="https://github.com/Mohamed0Hussein?tab=repositories"
            target="_blank"
            rel="noopener noreferrer"
            className="btn-outline"
            style={{ borderRadius: '14px' }}
          >
            View All on GitHub
            <ArrowUpRight className="w-4 h-4" />
          </Link>
        </motion.div>
      </div>
    </section>
  );
}

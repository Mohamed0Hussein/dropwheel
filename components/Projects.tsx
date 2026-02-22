'use client';

import { motion, useMotionTemplate, useMotionValue } from 'framer-motion';
import { ArrowUpRight, Github } from 'lucide-react';
import Link from 'next/link';
import { MouseEvent } from 'react';

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
  const mouseX = useMotionValue(0);
  const mouseY = useMotionValue(0);

  function handleMouseMove({ currentTarget, clientX, clientY }: MouseEvent) {
    const { left, top } = currentTarget.getBoundingClientRect();
    mouseX.set(clientX - left);
    mouseY.set(clientY - top);
  }

  const backgroundGlow = useMotionTemplate`
    radial-gradient(
      600px circle at ${mouseX}px ${mouseY}px,
      var(--accent-glow),
      transparent 80%
    )
  `;

  return (
    <motion.div
      initial={{ opacity: 0, scale: 0.95, y: 30 }}
      whileInView={{ opacity: 1, scale: 1, y: 0 }}
      viewport={{ once: true, margin: '-50px' }}
      transition={{ duration: 0.6, delay: index * 0.1, ease: [0.25, 1, 0.5, 1] }}
      className="card group relative flex flex-col justify-between"
      onMouseMove={handleMouseMove}
    >
      {/* Spotlight Hover Effect */}
      <motion.div
        className="pointer-events-none absolute -inset-px rounded-xl opacity-0 transition duration-500 group-hover:opacity-100 z-0"
        style={{ background: backgroundGlow }}
      />

      {/* Gradient Header Bar */}
      <div 
        className="h-1.5 w-full absolute top-0 left-0 transition-transform duration-500 origin-left scale-x-100 group-hover:scale-x-105 z-10" 
        style={{ background: project.gradient }} 
      />

      <div className="p-7 md:p-8 pt-10 grow z-20 relative bg-background/5 backdrop-blur-sm">
        {/* Category + Featured Tag */}
        <div className="flex items-center justify-between mb-5">
          <span
            className="text-xs font-bold uppercase tracking-[0.2em]"
            style={{ color: 'var(--text-tertiary)' }}
          >
            {project.subtitle}
          </span>
          {project.featured && (
            <motion.span
              whileHover={{ scale: 1.05 }}
              className="text-xs font-bold px-3 py-1.5 rounded-full shadow-sm"
              style={{
                background: 'var(--badge-bg)',
                color: 'var(--badge-text)',
                border: '1px solid var(--badge-border)',
                backdropFilter: 'blur(10px)',
              }}
            >
              Featured
            </motion.span>
          )}
        </div>

        {/* Title */}
        <h3
          className="text-2xl font-display font-bold mb-4 transition-colors duration-300 group-hover:text-accent group-hover:drop-shadow-sm"
          style={{ color: 'var(--text-primary)', letterSpacing: '-0.02em' }}
        >
          {project.title}
        </h3>

        {/* Description */}
        <p
          className="text-[0.95rem] leading-[1.6] mb-8 font-medium"
          style={{ color: 'var(--text-secondary)' }}
        >
          {project.description}
        </p>

        {/* Tech Stack */}
        <div className="flex flex-wrap gap-2.5 mb-8">
          {project.tech.map((tech) => (
            <span
              key={tech}
              className="text-xs font-semibold px-3 py-1.5 rounded-lg border transition-all duration-300 group-hover:border-accent/40"
              style={{
                background: 'var(--surface-elevated)',
                borderColor: 'var(--border-strong)',
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
          className="flex items-center gap-4 pt-5 mt-auto relative z-20"
          style={{ borderTop: '1px solid var(--border-strong)' }}
        >
          {project.live && (
            <Link
              href={project.live}
              target="_blank"
              rel="noopener noreferrer"
              className="btn-primary py-2 px-5 text-sm rounded-xl gap-2 shadow-sm"
            >
              Live Demo
              <ArrowUpRight className="w-4 h-4 transition-transform group-hover:translate-x-0.5 group-hover:-translate-y-0.5" />
            </Link>
          )}
          {project.github && (
            <Link
              href={project.github}
              target="_blank"
              rel="noopener noreferrer"
              className="btn-outline py-2 px-5 text-sm rounded-xl gap-2"
            >
              <Github className="w-4 h-4" />
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
    <section id="projects" className="py-24 md:py-32 px-6 md:px-8 relative z-10">
      <div className="max-w-6xl mx-auto relative z-10">
        <motion.div
          initial={{ opacity: 0, filter: 'blur(10px)', y: 24 }}
          whileInView={{ opacity: 1, filter: 'blur(0px)', y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.8, ease: [0.25, 1, 0.5, 1] }}
          className="mb-14"
        >
          <span className="section-label group">
            <span className="inline-block w-2h-2 rounded-full bg-accent mr-3 animate-pulse"></span>
            Portfolio
          </span>
          <h2 className="section-title text-5xl md:text-6xl lg:text-7xl mb-6 tracking-tighter shadow-sm">
            Selected work<span className="text-accent">.</span>
          </h2>
          <p className="section-subtitle font-light md:text-xl">
            Projects that showcase my ability to ship polished, production-grade applications that blend form and function perfectly.
          </p>
        </motion.div>

        {/* Projects Grid */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8 md:gap-10">
          {projects.map((project, index) => (
            <ProjectCard key={project.title} project={project} index={index} />
          ))}
        </div>

        {/* View All */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ delay: 0.5, duration: 0.6 }}
          className="mt-16 text-center"
        >
          <Link
            href="https://github.com/Mohamed0Hussein?tab=repositories"
            target="_blank"
            rel="noopener noreferrer"
            className="btn-outline px-8 py-4 rounded-full text-base"
          >
            Explore all repositories
            <ArrowUpRight className="w-5 h-5 ml-2" />
          </Link>
        </motion.div>
      </div>
    </section>
  );
}

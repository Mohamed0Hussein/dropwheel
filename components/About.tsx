'use client';

import { motion, Variants } from 'framer-motion';
import { Code2, Layers, Zap, Sparkles } from 'lucide-react';
import AnimatedCounter from './AnimatedCounter';

const highlights = [
  {
    icon: Code2,
    title: 'Clean Architecture',
    description:
      'Scalable, maintainable codebases built with TypeScript and advanced design patterns that stand the test of time.',
    color: '#7c3aed',
    gradient: 'linear-gradient(135deg, #7c3aed, #3b82f6)',
    bgGradient: 'radial-gradient(circle at 30% 30%, rgba(124,58,237,0.12) 0%, transparent 70%)',
  },
  {
    icon: Layers,
    title: 'Pixel-Perfect UI',
    description:
      '100% design fidelity translated carefully down to the very last pixel, transition delay, and easing curve.',
    color: '#3b82f6',
    gradient: 'linear-gradient(135deg, #3b82f6, #22d3ee)',
    bgGradient: 'radial-gradient(circle at 70% 30%, rgba(59,130,246,0.12) 0%, transparent 70%)',
  },
  {
    icon: Zap,
    title: 'Performance-First',
    description:
      'Relentlessly optimized rendering and smart bundle splitting for instantaneous, high-Lighthouse-score experiences.',
    color: '#0891b2',
    gradient: 'linear-gradient(135deg, #0891b2, #22d3ee)',
    bgGradient: 'radial-gradient(circle at 50% 70%, rgba(8,145,178,0.12) 0%, transparent 70%)',
  },
];

type Metric =
  | { value: number; suffix: string; label: string; color: string }
  | { value: null; display: string; label: string; color: string };

const metrics: Metric[] = [
  { value: 2, suffix: '+', label: 'Years', color: '#7c3aed' },
  { value: 10, suffix: '+', label: 'Projects', color: '#3b82f6' },
  { value: null, display: '∞', label: 'Curiosity', color: '#22d3ee' },
];

const containerVariants: Variants = {
  hidden: {},
  visible: { transition: { staggerChildren: 0.16, delayChildren: 0.2 } },
};

const cardVariants: Variants = {
  hidden: { opacity: 0, x: 32, scale: 0.97 },
  visible: {
    opacity: 1,
    x: 0,
    scale: 1,
    transition: { duration: 0.7, ease: [0.25, 1, 0.5, 1] },
  },
};

export default function About() {
  return (
    <section id="about" className="py-28 md:py-40 px-6 md:px-8 relative z-10 overflow-hidden">

      {/* Section-level ambient blobs */}
      <div
        className="absolute -top-1/4 -left-1/4 w-[600px] h-[600px] rounded-full pointer-events-none -z-10"
        style={{
          background: 'radial-gradient(circle, var(--orb-1) 0%, transparent 70%)',
          filter: 'blur(80px)',
          opacity: 0.5,
        }}
      />
      <div
        className="absolute -bottom-1/4 -right-1/4 w-[500px] h-[500px] rounded-full pointer-events-none -z-10"
        style={{
          background: 'radial-gradient(circle, var(--orb-2) 0%, transparent 70%)',
          filter: 'blur(80px)',
          opacity: 0.4,
        }}
      />

      <div className="max-w-6xl mx-auto">

        {/* ── Section header ── */}
        <motion.div
          initial={{ opacity: 0, y: 28 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true, margin: '-100px' }}
          transition={{ duration: 0.9, ease: [0.25, 1, 0.5, 1] }}
          className="mb-20"
        >
          <span className="section-label">Profile</span>
          <h2 className="section-title text-5xl md:text-6xl lg:text-7xl mb-6 tracking-tighter">
            Engineer, not just
            <br className="hidden md:block" />
            <span className="text-gradient">a coder</span>
            <span style={{ color: 'var(--accent)' }}>.</span>
          </h2>
          <p className="section-subtitle font-light md:text-xl max-w-2xl">
            I don&apos;t just write lines of code — I meticulously solve high-impact user problems
            through thoughtful, creative engineering. My obsession is building digital products
            that feel instantaneous, accessible, and cinematic.
          </p>
        </motion.div>

        {/* ── Two-column layout ── */}
        <div className="grid grid-cols-1 lg:grid-cols-5 gap-16 lg:gap-20 relative">

          {/* ── Left: Story ── */}
          <motion.div
            initial={{ opacity: 0, y: 28 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true, margin: '-80px' }}
            transition={{ duration: 0.9, delay: 0.15, ease: [0.25, 1, 0.5, 1] }}
            className="lg:col-span-2 space-y-7"
          >
            <p
              className="text-lg md:text-xl leading-relaxed font-medium"
              style={{ color: 'var(--text-secondary)' }}
            >
              I&apos;m{' '}
              <span className="text-gradient font-bold">Mohamed Hussein</span>
              , a Frontend Engineer with 2+ years of relentless experience building
              high-traffic, production-grade web applications.
            </p>
            <p
              className="text-base md:text-lg leading-relaxed"
              style={{ color: 'var(--text-tertiary)', fontWeight: 400 }}
            >
              My deep specialisation lies in{' '}
              <span className="font-semibold" style={{ color: 'var(--text-primary)' }}>React</span>
              ,{' '}
              <span className="font-semibold" style={{ color: 'var(--text-primary)' }}>Next.js</span>
              ,{' '}
              <span className="font-semibold" style={{ color: 'var(--text-primary)' }}>TypeScript</span>
              , and{' '}
              <span className="font-semibold" style={{ color: 'var(--text-primary)' }}>Framer Motion</span>
              . From orchestrating design systems to squeezing every ounce of load performance,
              I care intensely about developer velocity and end-user joy.
            </p>

            {/* Pull quote with gradient accent */}
            <motion.div
              className="relative pl-6 py-4"
              style={{ borderLeft: '3px solid transparent', borderImage: 'linear-gradient(to bottom, var(--gradient-from), var(--gradient-to)) 1' }}
              whileHover={{ x: 4 }}
              transition={{ type: 'spring', stiffness: 400, damping: 20 }}
            >
              <Sparkles
                className="w-4 h-4 mb-2"
                style={{ color: 'var(--accent)', opacity: 0.7 }}
              />
              <p
                className="text-base italic leading-relaxed font-medium"
                style={{ color: 'var(--text-tertiary)' }}
              >
                &quot;Quality is a habit, not an act. Everything built well stands the test of
                time.&quot;
              </p>
            </motion.div>

            {/* Mini metric row */}
            <div className="flex gap-8 pt-4 flex-wrap">
              {metrics.map((m) => (
                <motion.div
                  key={m.label}
                  className="flex flex-col gap-1"
                  whileHover={{ y: -4, scale: 1.05 }}
                  transition={{ type: 'spring', stiffness: 400, damping: 20 }}
                >
                  <span
                    className="text-2xl font-black font-display tracking-tight bg-clip-text text-transparent"
                    style={{ backgroundImage: `linear-gradient(135deg, ${m.color}, var(--gradient-via))` }}
                  >
                    {m.value !== null ? (
                      <AnimatedCounter value={m.value} suffix={m.suffix} />
                    ) : (
                      m.display
                    )}
                  </span>
                  <span
                    className="text-[0.65rem] uppercase tracking-widest font-bold"
                    style={{ color: 'var(--text-tertiary)' }}
                  >
                    {m.label}
                  </span>
                </motion.div>
              ))}
            </div>
          </motion.div>

          {/* ── Right: Highlight cards ── */}
          <motion.div
            className="lg:col-span-3 space-y-4"
            variants={containerVariants}
            initial="hidden"
            whileInView="visible"
            viewport={{ once: true, margin: '-60px' }}
          >
            {highlights.map((item) => (
              <motion.div
                key={item.title}
                variants={cardVariants}
                className="highlight-card group flex items-start gap-5 cursor-default relative overflow-hidden"
              >
                {/* Background gradient on hover */}
                <div
                  className="absolute inset-0 opacity-0 transition-opacity duration-500 group-hover:opacity-100 pointer-events-none"
                  style={{ background: item.bgGradient }}
                />

                {/* Icon */}
                <div
                  className="shrink-0 w-12 h-12 rounded-xl flex items-center justify-center transition-all duration-500 group-hover:scale-110 shadow-sm relative z-10"
                  style={{
                    background: item.bgGradient,
                    border: `1px solid ${item.color}30`,
                    boxShadow: `0 4px 16px ${item.color}20`,
                  }}
                >
                  <item.icon className="w-5 h-5 transition-all duration-300" style={{ color: item.color }} />
                </div>

                <div className="flex-1 min-w-0 relative z-10">
                  <h3
                    className="text-lg font-display font-bold mb-1.5 tracking-tight transition-colors duration-300 group-hover:text-accent"
                    style={{ color: 'var(--text-primary)' }}
                  >
                    {item.title}
                  </h3>
                  <p
                    className="text-sm leading-relaxed"
                    style={{ color: 'var(--text-tertiary)' }}
                  >
                    {item.description}
                  </p>
                </div>

                {/* Animated left border */}
                <motion.div
                  className="absolute left-0 top-0 bottom-0 w-[3px] rounded-full"
                  style={{ background: item.gradient }}
                  initial={{ scaleY: 0, originY: 0.5 }}
                  whileHover={{ scaleY: 1 }}
                  transition={{ duration: 0.35, ease: [0.25, 1, 0.5, 1] }}
                />
              </motion.div>
            ))}
          </motion.div>
        </div>
      </div>
    </section>
  );
}

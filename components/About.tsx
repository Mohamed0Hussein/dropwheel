'use client';

import { motion } from 'framer-motion';
import { Code2, Layers, Zap } from 'lucide-react';

const highlights = [
  {
    icon: Code2,
    title: 'Clean Architecture',
    description:
      'Scalable, maintainable codebases built with TypeScript and advanced design patterns that stand the test of time.',
    color: '#6366f1',
  },
  {
    icon: Layers,
    title: 'Pixel-Perfect UI',
    description:
      '100% design fidelity translated carefully down to the very last pixel, transition delay, and easing curve.',
    color: '#8b5cf6',
  },
  {
    icon: Zap,
    title: 'Performance-First',
    description:
      'Relentlessly optimized rendering and smart bundle splitting for instantaneous, high-Lighthouse-score experiences.',
    color: '#ec4899',
  },
];

const containerVariants = {
  hidden:  {},
  visible: { transition: { staggerChildren: 0.15, delayChildren: 0.2 } },
};

const cardVariants = {
  hidden:  { opacity: 0, x: 24 },
  visible: { opacity: 1, x: 0, transition: { duration: 0.65, ease: [0.25, 1, 0.5, 1] } },
};

export default function About() {
  return (
    <section id="about" className="py-24 md:py-36 px-6 md:px-8 relative z-10">
      <div className="max-w-6xl mx-auto">

        {/* ── Section header ── */}
        <motion.div
          initial={{ opacity: 0, y: 24 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true, margin: '-100px' }}
          transition={{ duration: 0.8, ease: [0.25, 1, 0.5, 1] }}
          className="mb-20"
        >
          <span className="section-label">Profile</span>
          <h2 className="section-title text-5xl md:text-6xl lg:text-7xl mb-5 tracking-tighter">
            Engineer, not just
            <br className="hidden md:block" />
            a coder<span className="text-accent">.</span>
          </h2>
          <p className="section-subtitle font-light md:text-xl max-w-2xl">
            I don&apos;t just write lines of code — I meticulously solve high-impact user problems
            through thoughtful, creative engineering. My obsession is building digital products
            that feel instantaneous, accessible, and cinematic.
          </p>
        </motion.div>

        {/* ── Two-column layout ── */}
        <div className="grid grid-cols-1 lg:grid-cols-5 gap-16 lg:gap-20 relative">

          {/* Ambient blob */}
          <div className="absolute top-0 right-0 w-80 h-80 rounded-full blur-3xl pointer-events-none -z-10"
            style={{ background: 'var(--mesh-2)', opacity: 0.6 }}
          />

          {/* ── Left: Story ── */}
          <motion.div
            initial={{ opacity: 0, y: 24 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true, margin: '-80px' }}
            transition={{ duration: 0.8, delay: 0.15, ease: [0.25, 1, 0.5, 1] }}
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
              className="text-base md:text-lg leading-relaxed font-light"
              style={{ color: 'var(--text-tertiary)' }}
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

            {/* Pull quote */}
            <div
              className="relative pl-5 py-1"
              style={{ borderLeft: '2px solid var(--accent)' }}
            >
              <div
                className="absolute -left-[5px] top-0 w-2 h-2 rounded-full"
                style={{ background: 'var(--accent)' }}
              />
              <p
                className="text-base italic leading-relaxed"
                style={{ color: 'var(--text-tertiary)' }}
              >
                &quot;Quality is a habit, not an act. Everything built well stands the test of
                time.&quot;
              </p>
            </div>

            {/* Mini metric row */}
            <div className="flex gap-8 pt-4">
              {[
                { n: '2+',  l: 'Years' },
                { n: '10+', l: 'Projects' },
                { n: '∞',   l: 'Curiosity' },
              ].map((m) => (
                <div key={m.l} className="flex flex-col gap-0.5">
                  <span
                    className="text-2xl font-black font-display text-gradient-static tracking-tight"
                  >
                    {m.n}
                  </span>
                  <span
                    className="text-[0.65rem] uppercase tracking-widest font-bold"
                    style={{ color: 'var(--text-tertiary)' }}
                  >
                    {m.l}
                  </span>
                </div>
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
                className="highlight-card group flex items-start gap-5 cursor-default relative"
              >
                {/* Icon */}
                <div
                  className="shrink-0 w-12 h-12 rounded-xl flex items-center justify-center transition-all duration-500 group-hover:scale-110 shadow-sm"
                  style={{
                    background: `${item.color}15`,
                    border: `1px solid ${item.color}30`,
                  }}
                >
                  <item.icon className="w-5 h-5" style={{ color: item.color }} />
                </div>

                <div className="flex-1 min-w-0">
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

                {/* Hover accent line (left edge) */}
                <motion.div
                  className="absolute left-0 top-1/2 -translate-y-1/2 w-0.5 rounded-full"
                  style={{ background: item.color }}
                  initial={{ height: 0 }}
                  whileHover={{ height: 48 }}
                  transition={{ duration: 0.3, ease: [0.25, 1, 0.5, 1] }}
                />
              </motion.div>
            ))}
          </motion.div>
        </div>
      </div>
    </section>
  );
}

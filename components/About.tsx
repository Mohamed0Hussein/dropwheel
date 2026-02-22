'use client';

import { motion } from 'framer-motion';
import { Code2, Layers, Zap } from 'lucide-react';

const highlights = [
  {
    icon: Code2,
    title: 'Clean Architecture',
    description: 'Scalable, maintainable codebases built with TypeScript and advanced design patterns.',
  },
  {
    icon: Layers,
    title: 'Pixel-Perfect UI',
    description: '100% design fidelity translated carefully down to the very last pixel and transition delay.',
  },
  {
    icon: Zap,
    title: 'Performance-First',
    description: 'Relentlessly optimized rendering and smart bundle splitting for instantaneous user experiences.',
  },
];

export default function About() {
  return (
    <section id="about" className="py-24 md:py-36 px-6 md:px-8 relative z-10">
      <div className="max-w-6xl mx-auto">
        <motion.div
          initial={{ opacity: 0, scale: 0.98, y: 24 }}
          whileInView={{ opacity: 1, scale: 1, y: 0 }}
          viewport={{ once: true, margin: '-100px' }}
          transition={{ duration: 0.8, ease: [0.25, 1, 0.5, 1] }}
          className="mb-20"
        >
          <span className="section-label">
            <span className="inline-block w-2h-2 rounded-full bg-accent mr-3 animate-pulse"></span>
            Profile
          </span>
          <h2 className="section-title text-5xl md:text-6xl lg:text-7xl mb-6 tracking-tighter shadow-sm">
            Engineer, not just <br className="hidden md:block"/>a coder<span className="text-accent">.</span>
          </h2>
          <p className="section-subtitle font-light md:text-xl">
            I don&apos;t just write lines of code — I meticulously solve high-impact user problems through thoughtful, creative engineering. My obsession is building digital products that feel instantaneous, accessible, and cinematic.
          </p>
        </motion.div>

        {/* Two-Column Layout */}
        <div className="grid grid-cols-1 lg:grid-cols-5 gap-16 lg:gap-20 relative">
          
          {/* Subtle Graphic background behind About text */}
          <div className="absolute top-0 right-10 w-96 h-96 bg-accent opacity-5 dark:opacity-10 rounded-full blur-3xl pointer-events-none -z-10" />

          {/* Left: Story */}
          <motion.div
            initial={{ opacity: 0, y: 24 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true, margin: '-100px' }}
            transition={{ duration: 0.8, delay: 0.2, ease: [0.25, 1, 0.5, 1] }}
            className="lg:col-span-2 space-y-8"
          >
            <p
              className="text-lg md:text-xl leading-relaxed font-medium"
              style={{ color: 'var(--text-secondary)' }}
            >
              I&apos;m{' '}
              <span className="text-gradient">
                Mohamed Hussein
              </span>
              , a Frontend Engineer with 2+ years of relentless experience building 
              high-traffic, production-grade web applications for dynamic products everywhere.
            </p>
            <p
              className="text-base md:text-lg leading-relaxed font-light"
              style={{ color: 'var(--text-tertiary)' }}
            >
              My deep specialization lies in{' '}
              <span className="font-semibold" style={{ color: 'var(--text-primary)' }}>React</span>,{' '}
              <span className="font-semibold" style={{ color: 'var(--text-primary)' }}>Next.js</span>,{' '}
              <span className="font-semibold" style={{ color: 'var(--text-primary)' }}>TypeScript</span>
              , and{' '}
              <span className="font-semibold" style={{ color: 'var(--text-primary)' }}>Framer Motion</span>. 
              From orchestrating comprehensive design systems and highly interactive UI components to squeezing every ounce of load performance, I care intensely about developer velocity and end-user joy.
            </p>
            <div className="pt-4 align-middle">
              <span className="inline-flex h-full w-0.5 bg-accent/20 rounded mr-6" />
              <p
                className="inline-block align-top max-w-[90%] text-base md:text-lg italic leading-relaxed"
                style={{ color: 'var(--text-tertiary)' }}
              >
                &quot;Quality is a habit, not an act. Everything built well stands the test of time.&quot;
              </p>
            </div>
          </motion.div>

          {/* Right: Highlight Cards */}
          <div className="lg:col-span-3 space-y-5">
            {highlights.map((item, index) => (
              <motion.div
                key={item.title}
                initial={{ opacity: 0, x: 20 }}
                whileInView={{ opacity: 1, x: 0 }}
                viewport={{ once: true, margin: '-50px' }}
                transition={{ duration: 0.6, delay: 0.3 + index * 0.15, ease: [0.25, 1, 0.5, 1] }}
                className="card flex flex-col sm:flex-row items-start gap-6 p-7 md:p-8 cursor-default group"
              >
                <div
                  className="shrink-0 w-14 h-14 rounded-2xl flex items-center justify-center transition-all duration-500 shadow-sm"
                  style={{
                    background: 'var(--surface-elevated)',
                    border: '1px solid var(--border)',
                  }}
                >
                  <item.icon
                    className="w-6 h-6 transition-all duration-500 scale-100 group-hover:scale-110 drop-shadow-sm"
                    style={{ color: 'var(--accent)' }}
                  />
                </div>
                <div>
                  <h3
                    className="text-xl font-display font-bold mb-2.5 transition-colors duration-300"
                    style={{ color: 'var(--text-primary)' }}
                  >
                    {item.title}
                  </h3>
                  <p
                    className="text-sm md:text-base leading-relaxed font-medium"
                    style={{ color: 'var(--text-tertiary)' }}
                  >
                    {item.description}
                  </p>
                </div>
                <div className="absolute inset-0 border border-transparent group-hover:border-accent/30 rounded-2xl pointer-events-none transition-colors duration-500"></div>
              </motion.div>
            ))}
          </div>
        </div>
      </div>
    </section>
  );
}

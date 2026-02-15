'use client';

import { motion } from 'framer-motion';
import { Code2, Layers, Zap } from 'lucide-react';

const highlights = [
  {
    icon: Code2,
    title: 'Clean Architecture',
    description: 'Scalable, maintainable codebases built with TypeScript and modern design patterns.',
  },
  {
    icon: Layers,
    title: 'Pixel-Perfect UI',
    description: '100% design fidelity with attention to every detail, spacing, and interaction.',
  },
  {
    icon: Zap,
    title: 'Performance-First',
    description: 'Optimized rendering, lazy loading, and bundle splitting for fast experiences.',
  },
];

export default function About() {
  return (
    <section id="about" className="py-24 md:py-32 px-6 md:px-8">
      <div className="max-w-6xl mx-auto">
        <motion.div
          initial={{ opacity: 0, y: 24 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true, margin: '-100px' }}
          transition={{ duration: 0.6 }}
          className="mb-16"
        >
          <span className="section-label">About</span>
          <h2 className="section-title text-4xl md:text-5xl lg:text-6xl mb-6">
            Engineer, not just a coder.
          </h2>
          <p className="section-subtitle">
            I don&apos;t just write code — I solve business problems through thoughtful engineering. 
            My focus is on building products that are fast, accessible, and a joy to use.
          </p>
        </motion.div>

        {/* Two-Column Layout */}
        <div className="grid grid-cols-1 lg:grid-cols-5 gap-12 lg:gap-16">
          {/* Left: Story */}
          <motion.div
            initial={{ opacity: 0, y: 24 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true, margin: '-100px' }}
            transition={{ duration: 0.6, delay: 0.1 }}
            className="lg:col-span-2 space-y-6"
          >
            <p
              className="text-lg leading-relaxed"
              style={{ color: 'var(--text-secondary)' }}
            >
              I&apos;m{' '}
              <span style={{ color: 'var(--text-primary)', fontWeight: 600 }}>
                Mohamed Hussein
              </span>
              , a Frontend Engineer with 2+ years of experience building 
              production-grade applications for companies across multiple industries.
            </p>
            <p
              className="leading-relaxed"
              style={{ color: 'var(--text-tertiary)' }}
            >
              I specialize in{' '}
              <span style={{ color: 'var(--text-secondary)' }}>React</span>,{' '}
              <span style={{ color: 'var(--text-secondary)' }}>Next.js</span>,{' '}
              <span style={{ color: 'var(--text-secondary)' }}>TypeScript</span>
              , and{' '}
              <span style={{ color: 'var(--text-secondary)' }}>Tailwind CSS</span>. 
              From building reusable component libraries to optimizing 
              build pipelines, I care deeply about developer experience 
              and end-user performance.
            </p>
            <p
              className="leading-relaxed"
              style={{ color: 'var(--text-tertiary)' }}
            >
              Beyond code, I thrive in collaborative environments where 
              design and engineering work hand-in-hand to ship products 
              that users love.
            </p>
          </motion.div>

          {/* Right: Highlight Cards */}
          <div className="lg:col-span-3 space-y-4">
            {highlights.map((item, index) => (
              <motion.div
                key={item.title}
                initial={{ opacity: 0, y: 20 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true, margin: '-50px' }}
                transition={{ duration: 0.5, delay: 0.15 + index * 0.1 }}
                className="card flex items-start gap-5 p-6 group cursor-default"
              >
                <div
                  className="shrink-0 w-10 h-10 rounded-xl flex items-center justify-center transition-colors duration-300"
                  style={{
                    background: 'var(--badge-bg)',
                    border: '1px solid var(--badge-border)',
                  }}
                >
                  <item.icon
                    className="w-5 h-5 transition-colors duration-300"
                    style={{ color: 'var(--accent)' }}
                  />
                </div>
                <div>
                  <h3
                    className="font-semibold mb-1.5"
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
              </motion.div>
            ))}
          </div>
        </div>
      </div>
    </section>
  );
}

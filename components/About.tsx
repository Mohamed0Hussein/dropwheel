'use client';

import { motion } from 'framer-motion';

export default function About() {
  return (
    <section id="about-me" className="py-32 px-6 md:px-16 lg:px-24">
      <div className="max-w-6xl mx-auto">
        <motion.div
          initial={{ opacity: 0, y: 30 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.6 }}
          className="grid grid-cols-1 lg:grid-cols-2 gap-16"
        >
          <div>
            <span className="text-sm uppercase tracking-widest mb-4 block" style={{ color: 'var(--text-secondary)' }}>About</span>
            <h2 className="font-display text-5xl md:text-6xl lg:text-7xl leading-tight" style={{ color: 'var(--text-primary)' }}>
              This is me
            </h2>
          </div>

          <div className="flex flex-col justify-center">
            <p className="text-xl md:text-2xl leading-relaxed mb-8" style={{ color: 'var(--text-secondary)' }}>
              I&apos;m a <span className="text-gradient font-semibold">Frontend Engineer</span> with over 2 years of experience building high-quality web applications.
            </p>
            <p className="leading-relaxed mb-8" style={{ color: 'var(--text-tertiary)' }}>
              I specialize in <span style={{ color: 'var(--text-secondary)' }}>React</span>, <span style={{ color: 'var(--text-secondary)' }}>Next.js</span>, <span style={{ color: 'var(--text-secondary)' }}>TypeScript</span>, and <span style={{ color: 'var(--text-secondary)' }}>Tailwind CSS</span>. I focus on creating scalable architecture, pixel-perfect interfaces, and seamless user experiences. My expertise extends to performance optimization, internationalization, and building reusable component libraries.
            </p>
            <p className="leading-relaxed" style={{ color: 'var(--text-tertiary)' }}>
              When I&apos;m not coding, you&apos;ll find me exploring new technologies, contributing to open-source projects, or sharing knowledge with the developer community.
            </p>
          </div>
        </motion.div>
      </div>
    </section>
  );
}

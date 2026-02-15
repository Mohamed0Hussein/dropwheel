'use client';

import { motion } from 'framer-motion';
import { Building2 } from 'lucide-react';

const experiences = [
  {
    period: 'Dec 2023 — Present',
    role: 'Frontend React Developer',
    company: 'Soshace',
    type: 'Full-time',
    achievements: [
      'Developed pixel-perfect UI components with 100% design fidelity across 20+ screens',
      'Led migration from legacy tooling to Vite, achieving 30% faster build times',
      'Identified and resolved critical production bugs, improving app stability by 40%',
      'Built reusable component library reducing development time for new features by 25%',
    ],
  },
  {
    period: 'Aug 2019 — Sept 2019',
    role: 'React Native Developer',
    company: 'Envent (prev. Codelabs)',
    type: 'Internship',
    achievements: [
      'Built a fully functional e-commerce mobile prototype from scratch',
      'Optimized UI responsiveness and rendering performance using lazy loading',
      'Collaborated with designers to implement complex animations and transitions',
    ],
  },
  {
    period: 'July 2017 — Aug 2017',
    role: 'WebGL Developer',
    company: 'iHub',
    type: 'Internship',
    achievements: [
      'Developed a 3D Kart Racing game using Babylon.js with interactive controls',
      'Optimized render pipeline, reducing frame lag by 40%',
      'Implemented real-time physics and collision detection systems',
    ],
  },
];

export default function Experience() {
  return (
    <section id="experience" className="py-24 md:py-32 px-6 md:px-8">
      <div className="max-w-6xl mx-auto">
        <motion.div
          initial={{ opacity: 0, y: 24 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.6 }}
          className="mb-12"
        >
          <span className="section-label">Career</span>
          <h2 className="section-title text-4xl md:text-5xl lg:text-6xl mb-4">
            Experience
          </h2>
          <p className="section-subtitle">
            Where I&apos;ve contributed and the impact I&apos;ve made.
          </p>
        </motion.div>

        {/* Timeline */}
        <div className="relative pl-8 md:pl-10">
          {/* Vertical Line */}
          <div className="timeline-line" />

          <div className="space-y-10">
            {experiences.map((exp, index) => (
              <motion.div
                key={exp.company}
                initial={{ opacity: 0, y: 24 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true, margin: '-50px' }}
                transition={{ duration: 0.5, delay: index * 0.1 }}
                className="relative"
              >
                {/* Timeline Dot */}
                <div
                  className="absolute -left-8 md:-left-10 top-1.5"
                  style={{ left: 'calc(-2rem + 9px)' }}
                >
                  <div className="timeline-dot" />
                </div>

                {/* Content Card */}
                <div className="card p-6 md:p-7">
                  {/* Header row */}
                  <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-2 mb-4">
                    <div>
                      <h3
                        className="text-lg font-bold"
                        style={{ color: 'var(--text-primary)', letterSpacing: '-0.01em' }}
                      >
                        {exp.role}
                      </h3>
                      <div className="flex items-center gap-2 mt-1">
                        <Building2 className="w-3.5 h-3.5" style={{ color: 'var(--accent)' }} />
                        <span
                          className="text-sm font-medium text-gradient-static"
                        >
                          {exp.company}
                        </span>
                      </div>
                    </div>
                    <div className="flex items-center gap-3">
                      <span
                        className="text-xs font-medium px-2.5 py-1 rounded-md"
                        style={{
                          background: 'var(--badge-bg)',
                          color: 'var(--badge-text)',
                          border: '1px solid var(--badge-border)',
                        }}
                      >
                        {exp.type}
                      </span>
                      <span
                        className="text-xs font-medium whitespace-nowrap"
                        style={{ color: 'var(--text-tertiary)' }}
                      >
                        {exp.period}
                      </span>
                    </div>
                  </div>

                  {/* Achievements */}
                  <ul className="space-y-2.5">
                    {exp.achievements.map((achievement, i) => (
                      <li
                        key={i}
                        className="flex items-start gap-3 text-sm leading-relaxed"
                        style={{ color: 'var(--text-secondary)' }}
                      >
                        <span
                          className="shrink-0 w-1.5 h-1.5 rounded-full mt-2"
                          style={{ background: 'var(--accent)' }}
                        />
                        {achievement}
                      </li>
                    ))}
                  </ul>
                </div>
              </motion.div>
            ))}
          </div>
        </div>
      </div>
    </section>
  );
}

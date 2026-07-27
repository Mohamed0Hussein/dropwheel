'use client';

import { motion } from 'framer-motion';
import { Building2, Calendar, Briefcase, TrendingUp } from 'lucide-react';

const experiences = [
  {
    period: 'Dec 2023 — Present',
    role: 'Frontend React Developer',
    company: 'Soshace',
    type: 'Full-time',
    current: true,
    gradient: 'from-indigo-500 to-violet-500',
    achievements: [
      'Engineered pixel-perfect UI components with 100% design fidelity architecture across 20+ complex screens.',
      'Led migration from legacy tooling to Vite, achieving 30% faster build times and improving HMR.',
      'Identified and resolved critical production bugs, elevating overall app stability by 40%.',
      'Architected a cohesive reusable component library, slashing development time for new features by 25%.',
    ],
  },
  {
    period: 'Aug 2019 — Sept 2019',
    role: 'React Native Developer',
    company: 'Envent (prev. Codelabs)',
    type: 'Internship',
    current: false,
    gradient: 'from-amber-500 to-orange-500',
    achievements: [
      'Single-handedly built a fully functional e-commerce mobile prototype from scratch.',
      'Optimized UI responsiveness and rendering performance integrating lazy loading and caching.',
      'Collaborated seamlessly with designers to implement fluid animations and spatial transitions.',
    ],
  },
  {
    period: 'July 2017 — Aug 2017',
    role: 'WebGL Developer',
    company: 'iHub',
    type: 'Internship',
    current: false,
    gradient: 'from-cyan-500 to-blue-500',
    achievements: [
      'Developed an immersive 3D Kart Racing game using Babylon.js with highly interactive controls.',
      'Heavily optimized the render pipeline, reducing overall frame lag by 40%.',
      'Implemented custom real-time physics and rigorous collision detection systems.',
    ],
  },
];

export default function Experience() {
  return (
    <section id="experience" className="py-24 md:py-36 px-6 md:px-8 relative z-10">
      <div className="max-w-6xl mx-auto">

        {/* Header */}
        <motion.div
          initial={{ opacity: 0, y: 24 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true, margin: '-100px' }}
          transition={{ duration: 0.8, ease: [0.25, 1, 0.5, 1] }}
          className="mb-20"
        >
          <span className="section-label">Career</span>
          <h2 className="section-title text-5xl md:text-6xl lg:text-7xl mb-5 tracking-tighter">
            Experience<span className="text-accent">.</span>
          </h2>
          <p className="section-subtitle font-light md:text-xl">
            A track record of shipping impactful software, optimizing architecture, and driving results.
          </p>
        </motion.div>

        {/* Timeline */}
        <div className="relative pl-10 md:pl-16">

          {/* Animated vertical line */}
          <motion.div
            initial={{ scaleY: 0 }}
            whileInView={{ scaleY: 1 }}
            viewport={{ once: true }}
            transition={{ duration: 1.6, ease: 'easeInOut' }}
            className="timeline-line hidden md:block origin-top"
            style={{ left: 'clamp(17px, 4vw, 26px)' }}
          />

          <div className="space-y-12">
            {experiences.map((exp, index) => (
              <motion.div
                key={exp.company}
                initial={{ opacity: 0, x: -32 }}
                whileInView={{ opacity: 1, x: 0 }}
                viewport={{ once: true, margin: '-80px' }}
                transition={{ duration: 0.7, delay: index * 0.15, ease: [0.25, 1, 0.5, 1] }}
                className="relative group md:pl-10"
              >
                {/* Timeline dot */}
                <div className="absolute hidden md:flex items-center justify-center top-7 -left-[calc(2.5rem+1px)] md:-left-[calc(3.5rem+1px)] z-10">
                  <motion.div
                    className="timeline-dot group-hover:scale-125 transition-transform duration-400"
                    whileInView={{ scale: [0, 1.2, 1] }}
                    viewport={{ once: true }}
                    transition={{ duration: 0.5, delay: index * 0.15 + 0.3 }}
                  />
                </div>

                {/* Card */}
                <div className="card p-7 md:p-9 relative overflow-hidden hover:-translate-y-1.5 hover:shadow-2xl transition-all duration-400">

                  {/* Top gradient accent */}
                  <div
                    className={`absolute top-0 left-0 right-0 h-[3px] bg-gradient-to-r ${exp.gradient} opacity-60 group-hover:opacity-100 transition-opacity duration-400`}
                  />

                  {/* Ambient blob */}
                  <div className="absolute top-0 right-0 w-56 h-56 rounded-full blur-3xl -z-10 opacity-0 group-hover:opacity-100 transition-opacity duration-500"
                    style={{ background: `var(--accent-light)` }}
                  />

                  {/* Header row */}
                  <div className="flex flex-col xl:flex-row xl:items-start justify-between gap-4 mb-7">
                    <div className="flex-1">
                      {/* Role + current badge */}
                      <div className="flex flex-wrap items-center gap-3 mb-3">
                        <h3
                          className="text-xl md:text-2xl font-display font-bold tracking-tight group-hover:text-accent transition-colors duration-300"
                          style={{ color: 'var(--text-primary)' }}
                        >
                          {exp.role}
                        </h3>
                        {exp.current && (
                          <span
                            className="inline-flex items-center gap-1.5 text-[0.65rem] font-bold uppercase tracking-widest px-2.5 py-1 rounded-full"
                            style={{
                              background: 'rgba(34,197,94,0.1)',
                              color: '#22c55e',
                              border: '1px solid rgba(34,197,94,0.25)',
                            }}
                          >
                            <span className="w-1.5 h-1.5 rounded-full bg-green-400 animate-pulse" />
                            Current
                          </span>
                        )}
                      </div>

                      {/* Company + type row */}
                      <div className="flex flex-wrap items-center gap-4">
                        <div className="flex items-center gap-2">
                          <Building2 className="w-3.5 h-3.5" style={{ color: 'var(--accent)' }} />
                          <span className="text-sm font-bold text-gradient-static">
                            {exp.company}
                          </span>
                        </div>
                        <div className="flex items-center gap-1.5">
                          <Briefcase className="w-3.5 h-3.5" style={{ color: 'var(--text-tertiary)' }} />
                          <span
                            className="text-[0.65rem] font-bold uppercase tracking-widest px-2.5 py-1 rounded-full"
                            style={{
                              background: 'var(--badge-bg)',
                              color: 'var(--badge-text)',
                              border: '1px solid var(--badge-border)',
                            }}
                          >
                            {exp.type}
                          </span>
                        </div>
                      </div>
                    </div>

                    {/* Period */}
                    <div className="flex items-center gap-2 shrink-0">
                      <Calendar className="w-3.5 h-3.5" style={{ color: 'var(--text-tertiary)' }} />
                      <span
                        className="text-sm font-semibold whitespace-nowrap"
                        style={{ color: 'var(--text-secondary)' }}
                      >
                        {exp.period}
                      </span>
                    </div>
                  </div>

                  {/* Achievements */}
                  <ul
                    className="space-y-3 pt-5"
                    style={{ borderTop: '1px solid var(--border)' }}
                  >
                    {exp.achievements.map((item, i) => (
                      <motion.li
                        key={i}
                        initial={{ opacity: 0, x: -10 }}
                        whileInView={{ opacity: 1, x: 0 }}
                        viewport={{ once: true }}
                        transition={{ duration: 0.4, delay: index * 0.1 + i * 0.08 }}
                        className="flex items-start gap-3 text-[0.9rem] leading-relaxed group/item"
                        style={{ color: 'var(--text-secondary)' }}
                      >
                        <TrendingUp
                          className="w-3.5 h-3.5 shrink-0 mt-1 transition-colors duration-300 group-hover/item:text-accent"
                          style={{ color: 'var(--accent)', opacity: 0.7 }}
                        />
                        <span className="group-hover/item:text-text-primary transition-colors duration-300">
                          {item}
                        </span>
                      </motion.li>
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

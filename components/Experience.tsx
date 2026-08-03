'use client';

import { motion } from 'framer-motion';
import { Building2, Calendar, Briefcase } from 'lucide-react';
import ExpandableExperienceCard from './ExpandableExperienceCard';

const experiences = [
  {
    period: 'Dec 2023 — Present',
    role: 'Frontend React Developer',
    company: 'Soshace',
    type: 'Full-time',
    current: true,
    gradient: 'linear-gradient(135deg, #7c3aed, #3b82f6)',
    accentColor: '#7c3aed',
    dotColor: '#7c3aed',
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
    gradient: 'linear-gradient(135deg, #3b82f6, #0ea5e9)',
    accentColor: '#3b82f6',
    dotColor: '#3b82f6',
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
    gradient: 'linear-gradient(135deg, #0891b2, #22d3ee)',
    accentColor: '#0891b2',
    dotColor: '#0891b2',
    achievements: [
      'Developed an immersive 3D Kart Racing game using Babylon.js with highly interactive controls.',
      'Heavily optimized the render pipeline, reducing overall frame lag by 40%.',
      'Implemented custom real-time physics and rigorous collision detection systems.',
    ],
  },
];

export default function Experience() {
  return (
    <section id="experience" className="py-28 md:py-40 px-6 md:px-8 relative z-10 overflow-hidden">

      {/* Ambient blobs */}
      <div
        className="absolute top-0 right-0 w-[500px] h-[500px] rounded-full pointer-events-none -z-10"
        style={{
          background: 'radial-gradient(circle, var(--orb-2) 0%, transparent 70%)',
          filter: 'blur(100px)',
          opacity: 0.4,
        }}
      />

      <div className="max-w-6xl mx-auto">

        {/* Header */}
        <motion.div
          initial={{ opacity: 0, y: 28 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true, margin: '-100px' }}
          transition={{ duration: 0.9, ease: [0.25, 1, 0.5, 1] }}
          className="mb-20"
        >
          <span className="section-label">Career</span>
          <h2 className="section-title text-5xl md:text-6xl lg:text-7xl mb-5 tracking-tighter">
            Experience<span style={{ color: 'var(--accent)' }}>.</span>
          </h2>
          <p className="section-subtitle font-light md:text-xl">
            A track record of shipping impactful software, optimizing architecture, and driving results.
          </p>
        </motion.div>

        {/* Timeline */}
        <div className="relative pl-10 md:pl-20">

          {/* Animated vertical line */}
          <motion.div
            initial={{ scaleY: 0 }}
            whileInView={{ scaleY: 1 }}
            viewport={{ once: true }}
            transition={{ duration: 1.8, ease: 'easeInOut' }}
            className="timeline-line hidden md:block origin-top"
            style={{ left: 'clamp(18px, 4vw, 28px)' }}
          />

          <div className="space-y-10">
            {experiences.map((exp, index) => (
              <motion.div
                key={exp.company}
                initial={{ opacity: 0, x: -40 }}
                whileInView={{ opacity: 1, x: 0 }}
                viewport={{ once: true, margin: '-80px' }}
                transition={{ duration: 0.75, delay: index * 0.16, ease: [0.25, 1, 0.5, 1] }}
                className="relative group md:pl-12"
              >
                {/* Timeline dot */}
                <div className="absolute hidden md:flex items-center justify-center top-8 -left-[calc(3rem+1px)] md:-left-[calc(4rem+1px)] z-10">
                  <motion.div
                    className="relative"
                    whileInView={{ scale: [0, 1.3, 1] }}
                    viewport={{ once: true }}
                    transition={{ duration: 0.6, delay: index * 0.16 + 0.35 }}
                  >
                    {/* Outer ring */}
                    <motion.div
                      className="absolute inset-0 rounded-full"
                      style={{
                        background: exp.accentColor,
                        opacity: 0.15,
                      }}
                      animate={{ scale: [1, 1.8, 1], opacity: [0.15, 0, 0.15] }}
                      transition={{ duration: 3, repeat: Infinity, ease: 'easeInOut', delay: index * 0.5 }}
                    />
                    <div
                      className="relative w-3.5 h-3.5 rounded-full border-[3px] shadow-lg"
                      style={{
                        background: 'var(--background)',
                        borderColor: exp.accentColor,
                        boxShadow: `0 0 0 4px var(--background), 0 0 20px ${exp.accentColor}60`,
                      }}
                    />
                  </motion.div>
                </div>

                {/* Card */}
                <div
                  className="card p-7 md:p-9 relative overflow-hidden hover:-translate-y-2 transition-all duration-500 group"
                  style={{ '--accent-color': exp.accentColor } as any}
                >
                  {/* Top gradient accent — animated width on hover */}
                  <motion.div
                    className="absolute top-0 left-0 h-[3px] rounded-t-[20px]"
                    style={{ background: exp.gradient, right: 0 }}
                    initial={{ opacity: 0.5, scaleX: 0.7, originX: 0 }}
                    whileInView={{ opacity: 0.7, scaleX: 1 }}
                    whileHover={{ opacity: 1, scaleX: 1 }}
                    viewport={{ once: true }}
                    transition={{ duration: 0.6 }}
                  />

                  {/* Ambient glow in top-right */}
                  <div
                    className="absolute -top-20 -right-20 w-64 h-64 rounded-full pointer-events-none opacity-0 group-hover:opacity-100 transition-opacity duration-700"
                    style={{
                      background: `radial-gradient(circle, ${exp.accentColor}18 0%, transparent 70%)`,
                      filter: 'blur(20px)',
                    }}
                  />

                  {/* Header row */}
                  <div className="flex flex-col xl:flex-row xl:items-start justify-between gap-4 mb-7">
                    <div className="flex-1">
                      {/* Role + current badge */}
                      <div className="flex flex-wrap items-center gap-3 mb-3">
                        <h3
                          className="text-xl md:text-2xl font-display font-bold tracking-tight transition-colors duration-300"
                          style={{ color: 'var(--text-primary)', letterSpacing: '-0.03em' }}
                        >
                          {exp.role}
                        </h3>
                        {exp.current && (
                          <motion.span
                            className="inline-flex items-center gap-1.5 text-[0.65rem] font-bold uppercase tracking-widest px-2.5 py-1 rounded-full"
                            style={{
                              background: 'rgba(34,197,94,0.1)',
                              color: '#22c55e',
                              border: '1px solid rgba(34,197,94,0.25)',
                            }}
                            animate={{ opacity: [1, 0.7, 1] }}
                            transition={{ duration: 2.5, repeat: Infinity }}
                          >
                            <span className="w-1.5 h-1.5 rounded-full bg-green-400 animate-pulse" />
                            Current
                          </motion.span>
                        )}
                      </div>

                      {/* Company + type row */}
                      <div className="flex flex-wrap items-center gap-4">
                        <div className="flex items-center gap-2">
                          <Building2 className="w-3.5 h-3.5" style={{ color: exp.accentColor }} />
                          <span
                            className="text-sm font-bold"
                            style={{
                              background: exp.gradient,
                              WebkitBackgroundClip: 'text',
                              backgroundClip: 'text',
                              color: 'transparent',
                            }}
                          >
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
                  <ExpandableExperienceCard
                    achievements={exp.achievements}
                    accentColor={exp.accentColor}
                    baseDelay={index * 0.1}
                  />
                </div>
              </motion.div>
            ))}
          </div>
        </div>
      </div>
    </section>
  );
}

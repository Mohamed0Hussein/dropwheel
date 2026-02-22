'use client';

import { motion } from 'framer-motion';
import { Building2, Calendar, Briefcase } from 'lucide-react';

const experiences = [
  {
    period: 'Dec 2023 — Present',
    role: 'Frontend React Developer',
    company: 'Soshace',
    type: 'Full-time',
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
        <motion.div
          initial={{ opacity: 0, scale: 0.98, y: 24 }}
          whileInView={{ opacity: 1, scale: 1, y: 0 }}
          viewport={{ once: true, margin: '-100px' }}
          transition={{ duration: 0.8, ease: [0.25, 1, 0.5, 1] }}
          className="mb-20 text-center md:text-left"
        >
          <span className="section-label">
            <span className="inline-block w-2h-2 rounded-full bg-accent mr-3 animate-pulse"></span>
            Career
          </span>
          <h2 className="section-title text-5xl md:text-6xl lg:text-7xl mb-6 tracking-tighter">
            Experience<span className="text-accent">.</span>
          </h2>
          <p className="section-subtitle font-light md:text-xl">
            A track record of shipping impactful software, optimizing architecture, and driving results.
          </p>
        </motion.div>

        {/* Timeline */}
        <div className="relative pl-8 md:pl-12 lg:pl-16">
          {/* Vertical Line animated */}
          <motion.div 
            initial={{ height: 0 }}
            whileInView={{ height: '100%' }}
            viewport={{ once: true }}
            transition={{ duration: 1.5, ease: 'easeInOut' }}
            className="timeline-line hidden md:block" 
            style={{ left: 'clamp(17px, 5vw, 30px)' }}
          />

          <div className="space-y-16">
            {experiences.map((exp, index) => (
              <motion.div
                key={exp.company}
                initial={{ opacity: 0, x: -30 }}
                whileInView={{ opacity: 1, x: 0 }}
                viewport={{ once: true, margin: '-100px' }}
                transition={{ duration: 0.7, delay: index * 0.2, ease: [0.25, 1, 0.5, 1] }}
                className="relative group md:pl-8"
              >
                {/* Timeline Dot */}
                <div
                  className="absolute hidden md:block top-6 -left-[2.1rem] lg:-left-[2.9rem] z-10"
                >
                  <div className="timeline-dot transition-transform duration-500 group-hover:scale-125 group-hover:bg-accent" />
                </div>

                {/* Content Card */}
                <div className="card p-7 md:p-10 relative overflow-hidden transition-all duration-500 hover:shadow-2xl hover:-translate-y-2">
                  <div className="absolute top-0 right-0 w-64 h-64 bg-accent/5 rounded-full blur-3xl -z-10 group-hover:bg-accent/10 transition-colors duration-500"></div>
                  
                  {/* Header row */}
                  <div className="flex flex-col xl:flex-row xl:items-start justify-between gap-4 mb-8">
                    <div>
                      <h3
                        className="text-2xl font-display font-bold mb-3 group-hover:text-accent transition-colors duration-300"
                        style={{ color: 'var(--text-primary)', letterSpacing: '-0.02em' }}
                      >
                        {exp.role}
                      </h3>
                      <div className="flex flex-wrap items-center gap-4">
                        <div className="flex items-center gap-2">
                          <Building2 className="w-4 h-4 text-accent" />
                          <span className="text-base font-semibold text-gradient-static tracking-wide">
                            {exp.company}
                          </span>
                        </div>
                        <div className="flex items-center gap-2">
                          <Briefcase className="w-4 h-4 text-text-tertiary opacity-70" />
                          <span
                            className="text-xs font-bold uppercase tracking-widest px-3 py-1.5 rounded-full"
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
                    
                    <div className="flex items-center gap-2 mt-2 xl:mt-0 xl:justify-end">
                      <Calendar className="w-4 h-4 text-text-tertiary opacity-70" />
                      <span
                        className="text-sm font-semibold whitespace-nowrap"
                        style={{ color: 'var(--text-secondary)' }}
                      >
                        {exp.period}
                      </span>
                    </div>
                  </div>

                  {/* Achievements */}
                  <ul className="space-y-4 pt-4 border-t border-border/50">
                    {exp.achievements.map((achievement, i) => (
                      <li
                        key={i}
                        className="flex items-start gap-4 text-[0.95rem] leading-[1.6] group/item"
                        style={{ color: 'var(--text-secondary)' }}
                      >
                        <span
                          className="shrink-0 w-1.5 h-1.5 rounded-full mt-2.5 transition-all duration-300 group-hover/item:scale-150"
                          style={{ background: 'var(--accent)' }}
                        />
                        <span className="group-hover/item:text-text-primary transition-colors duration-300">{achievement}</span>
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

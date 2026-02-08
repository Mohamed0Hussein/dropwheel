'use client';

import { motion } from 'framer-motion';

const experiences = [
  {
    period: 'Dec 2023 - Present',
    role: 'Frontend React Developer',
    company: 'Soshace',
    description: 'Developed pixel-perfect UI components ensuring 100% design fidelity. Identified and resolved critical bugs, improving stability and performance. Led migration from outdated tools to modern frameworks (Vite), resulting in 30% faster development workflow.'
  },
  {
    period: 'Aug 2019 - Sept 2019',
    role: 'React Native Developer Intern',
    company: 'Envent (previously Codelabs)',
    description: 'Built a fully functional mobile e-commerce prototype, optimizing UI responsiveness. Improved React Native performance using lazy loading and optimized rendering techniques.'
  },
  {
    period: 'July 2017 - Aug 2017',
    role: 'WebGL Developer Intern',
    company: 'iHub',
    description: 'Developed a 3D Kart Racing Game using Babylon.js, enhancing interactive graphics. Optimized game performance, reducing render lag by 40%.'
  }
];

export default function Experience() {
  return (
    <section id="experience" className="py-32 px-6 md:px-16 lg:px-24">
      <div className="max-w-6xl mx-auto">
        <motion.div
          initial={{ opacity: 0, y: 30 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.6 }}
          className="mb-16"
        >
          <span className="text-sm uppercase tracking-widest mb-4 block" style={{ color: 'var(--text-secondary)' }}>Career</span>
          <h2 className="font-display text-5xl md:text-6xl lg:text-7xl" style={{ color: 'var(--text-primary)' }}>
            Experience
          </h2>
        </motion.div>

        <div className="relative">
          <div className="absolute left-0 md:left-1/2 top-0 w-px h-full transform md:-translate-x-1/2" style={{ background: 'var(--border-primary)' }} />

          <div className="space-y-12">
            {experiences.map((exp, index) => (
              <motion.div
                key={index}
                initial={{ opacity: 0, y: 30 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                transition={{ duration: 0.5, delay: index * 0.1 }}
                className={`relative flex flex-col md:flex-row gap-8 ${
                  index % 2 === 0 ? 'md:flex-row' : 'md:flex-row-reverse'
                }`}
              >
                <div className="absolute left-0 md:left-1/2 w-3 h-3 rounded-full transform -translate-x-1/2 mt-2 z-10" style={{ background: 'var(--text-primary)' }} />

                <div className={`md:w-1/2 pl-8 md:pl-0 ${index % 2 === 0 ? 'md:pr-16 md:text-right' : 'md:pl-16'}`}>
                  <span className="text-sm tracking-wider" style={{ color: 'var(--text-tertiary)' }}>{exp.period}</span>
                  <h3 className="text-2xl font-semibold mt-2" style={{ color: 'var(--text-primary)' }}>{exp.role}</h3>
                  <p className="text-lg mt-1 text-gradient font-medium">{exp.company}</p>
                  <p className="mt-4 leading-relaxed" style={{ color: 'var(--text-tertiary)' }}>{exp.description}</p>
                </div>

                <div className="hidden md:block md:w-1/2" />
              </motion.div>
            ))}
          </div>
        </div>
      </div>
    </section>
  );
}

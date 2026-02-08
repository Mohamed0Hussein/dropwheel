'use client';

import { motion } from 'framer-motion';
import Link from 'next/link';

const stats = [
  { value: '2+', label: 'Years of Experience' },
  { value: '10+', label: 'Projects Completed' },
  { value: '3', label: 'Companies' },
];

export default function Hero() {
  return (
    <section className="min-h-screen flex flex-col justify-center relative px-6 md:px-16 lg:px-24">
      <div className="max-w-6xl mx-auto w-full">
        <motion.div
          initial={{ opacity: 0, y: 40 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.8 }}
          className="mb-8"
        >
          <h1 className="font-display text-6xl sm:text-7xl md:text-8xl lg:text-[10rem] leading-[0.9] tracking-tight text-gradient">
            FRONTEND
            <br />
            DEVELOPER
          </h1>
        </motion.div>

        <motion.div
          initial={{ opacity: 0, y: 30 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.8, delay: 0.2 }}
          className="flex flex-col md:flex-row md:items-end justify-between gap-8 mb-12"
        >
          <div className="max-w-lg">
            <p className="text-lg leading-relaxed" style={{ color: 'var(--text-secondary)' }}>
              Hi, I&apos;m <span className="font-medium" style={{ color: 'var(--text-primary)' }}>Mohamed Hussein</span>, 
              a passionate developer creating exceptional digital experiences with modern web technologies.
            </p>
          </div>
          
          <Link
            href="https://www.linkedin.com/in/mohamed-hussein-0x1/"
            target="_blank"
            className="btn-primary inline-flex items-center gap-3 px-8 py-4 rounded-full font-semibold group self-start md:self-auto"
          >
            <span>Hire Me</span>
            <svg 
              className="w-5 h-5 transition-transform group-hover:translate-x-1" 
              fill="none" 
              viewBox="0 0 24 24" 
              stroke="currentColor"
            >
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M17 8l4 4m0 0l-4 4m4-4H3" />
            </svg>
          </Link>
        </motion.div>

        <motion.div
          initial={{ opacity: 0, y: 30 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.8, delay: 0.4 }}
          className="flex flex-wrap gap-8 md:gap-16 pt-8"
          style={{ borderTop: '1px solid var(--border-primary)' }}
        >
          {stats.map((stat, index) => (
            <div key={index} className="flex items-baseline gap-3">
              <span className="text-4xl md:text-5xl font-display text-gradient">{stat.value}</span>
              <span className="text-sm uppercase tracking-wider" style={{ color: 'var(--text-tertiary)' }}>{stat.label}</span>
            </div>
          ))}
        </motion.div>
      </div>

      <motion.div
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ delay: 1.5 }}
        className="absolute bottom-12 left-1/2 -translate-x-1/2 flex flex-col items-center gap-2"
      >
        <motion.div
          animate={{ y: [0, 10, 0] }}
          transition={{ duration: 2, repeat: Infinity }}
          className="w-6 h-10 rounded-full flex justify-center pt-2"
          style={{ border: '2px solid var(--border-primary)' }}
        >
          <motion.div
            animate={{ opacity: [0.5, 1, 0.5], y: [0, 12, 0] }}
            transition={{ duration: 2, repeat: Infinity }}
            className="w-1 h-2 rounded-full"
            style={{ background: 'var(--text-primary)' }}
          />
        </motion.div>
      </motion.div>
    </section>
  );
}

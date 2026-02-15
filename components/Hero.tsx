'use client';

import { motion } from 'framer-motion';
import Link from 'next/link';
import { ArrowRight, MapPin } from 'lucide-react';

const stats = [
  { value: '2+', label: 'Years Experience' },
  { value: '10+', label: 'Projects Shipped' },
  { value: '30%', label: 'Faster Dev Workflows' },
];

export default function Hero() {
  return (
    <section
      id="hero"
      className="min-h-screen flex flex-col justify-center relative px-6 md:px-8 pt-24"
    >
      <div className="max-w-6xl mx-auto w-full">
        {/* Status Badge */}
        <motion.div
          initial={{ opacity: 0, y: 16 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.1 }}
          className="mb-8"
        >
          <div
            className="inline-flex items-center gap-3 px-4 py-2.5 rounded-full text-sm font-medium"
            style={{
              background: 'var(--badge-bg)',
              border: '1px solid var(--badge-border)',
              color: 'var(--badge-text)',
            }}
          >
            <span className="status-dot" />
            Available for new opportunities
          </div>
        </motion.div>

        {/* Headline */}
        <motion.div
          initial={{ opacity: 0, y: 24 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.7, delay: 0.2 }}
          className="mb-6"
        >
          <h1 className="font-display text-5xl sm:text-6xl md:text-7xl lg:text-[5.5rem]">
            <span style={{ color: 'var(--text-primary)' }}>
              I build interfaces
            </span>
            <br />
            <span className="text-gradient">
              that perform.
            </span>
          </h1>
        </motion.div>

        {/* Subtitle */}
        <motion.div
          initial={{ opacity: 0, y: 24 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.7, delay: 0.35 }}
          className="mb-10 max-w-xl"
        >
          <p
            className="text-lg md:text-xl leading-relaxed"
            style={{ color: 'var(--text-secondary)' }}
          >
            Frontend Engineer crafting{' '}
            <span style={{ color: 'var(--text-primary)', fontWeight: 600 }}>
              high-performance React applications
            </span>{' '}
            with pixel-perfect precision and scalable architecture.
          </p>
        </motion.div>

        {/* CTA Buttons */}
        <motion.div
          initial={{ opacity: 0, y: 24 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.7, delay: 0.5 }}
          className="flex flex-wrap items-center gap-4 mb-20"
        >
          <Link href="#projects" className="btn-primary">
            View My Work
            <ArrowRight className="w-4 h-4" />
          </Link>
          <Link href="#contact" className="btn-outline">
            Hire Me
          </Link>
        </motion.div>

        {/* Stats + Location */}
        <motion.div
          initial={{ opacity: 0, y: 24 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.7, delay: 0.65 }}
          className="flex flex-wrap items-center gap-8 md:gap-12 pt-8"
          style={{ borderTop: '1px solid var(--border)' }}
        >
          {stats.map((stat) => (
            <div key={stat.label} className="flex items-baseline gap-2.5">
              <span className="text-3xl md:text-4xl font-bold text-gradient-static">
                {stat.value}
              </span>
              <span
                className="text-xs md:text-sm uppercase tracking-wider font-medium"
                style={{ color: 'var(--text-tertiary)' }}
              >
                {stat.label}
              </span>
            </div>
          ))}
          <div
            className="flex items-center gap-2 ml-auto"
            style={{ color: 'var(--text-tertiary)' }}
          >
            <MapPin className="w-4 h-4" />
            <span className="text-sm font-medium">Cairo, Egypt</span>
          </div>
        </motion.div>
      </div>

      {/* Scroll Indicator */}
      <motion.div
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ delay: 1.6 }}
        className="absolute bottom-10 left-1/2 -translate-x-1/2 flex flex-col items-center gap-2"
      >
        <span className="text-[10px] uppercase tracking-[0.15em] font-medium" style={{ color: 'var(--text-tertiary)' }}>
          Scroll
        </span>
        <motion.div
          animate={{ y: [0, 6, 0] }}
          transition={{ duration: 1.8, repeat: Infinity, ease: 'easeInOut' }}
          className="w-5 h-8 rounded-full flex justify-center pt-1.5"
          style={{ border: '1.5px solid var(--border-strong)' }}
        >
          <motion.div
            animate={{ opacity: [0.3, 1, 0.3], y: [0, 10, 0] }}
            transition={{ duration: 1.8, repeat: Infinity, ease: 'easeInOut' }}
            className="w-1 h-1.5 rounded-full"
            style={{ background: 'var(--accent)' }}
          />
        </motion.div>
      </motion.div>
    </section>
  );
}

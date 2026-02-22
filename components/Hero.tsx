'use client';

import { motion, useScroll, useTransform } from 'framer-motion';
import Link from 'next/link';
import { ArrowRight, MapPin } from 'lucide-react';
import { useRef } from 'react';

const stats = [
  { value: '2+', label: 'Years Experience' },
  { value: '10+', label: 'Projects Shipped' },
  { value: '30%', label: 'Faster Workflows' },
];

export default function Hero() {
  const containerRef = useRef<HTMLDivElement>(null);
  const { scrollYProgress } = useScroll({
    target: containerRef,
    offset: ['start start', 'end start'],
  });

  const y1 = useTransform(scrollYProgress, [0, 1], [0, 150]);
  const y2 = useTransform(scrollYProgress, [0, 1], [0, 100]);
  const opacity = useTransform(scrollYProgress, [0, 0.8], [1, 0]);

  return (
    <section
      id="hero"
      ref={containerRef}
      className="min-h-[100svh] flex flex-col justify-center relative px-6 md:px-8 pt-24 overflow-hidden"
    >
      <motion.div style={{ y: y1, opacity }} className="max-w-6xl mx-auto w-full relative z-10">
        {/* Status Badge */}
        <motion.div
          initial={{ opacity: 0, scale: 0.95 }}
          animate={{ opacity: 1, scale: 1 }}
          transition={{ duration: 0.6, ease: [0.25, 1, 0.5, 1] }}
          className="mb-8 inline-block"
        >
          <div
            className="inline-flex items-center gap-3 px-4 py-2.5 rounded-full text-sm font-medium backdrop-blur-md transition-all duration-300 hover:scale-105 cursor-default"
            style={{
              background: 'var(--badge-bg)',
              border: '1px solid var(--badge-border)',
              color: 'var(--badge-text)',
              boxShadow: '0 4px 12px var(--accent-light)',
            }}
          >
            <span className="status-dot" />
            <span className="tracking-tight">Available for new opportunities</span>
          </div>
        </motion.div>

        {/* Headline */}
        <motion.div
          initial={{ opacity: 0, y: 30 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.8, delay: 0.1, ease: [0.25, 1, 0.5, 1] }}
          className="mb-6 relative"
        >
          <h1 className="font-display text-[3.5rem] leading-[1.05] sm:text-7xl md:text-8xl lg:text-[6.5rem] tracking-tighter">
            <span className="block text-primary" style={{ color: 'var(--text-primary)' }}>
              I build interfaces
            </span>
            <span className="block text-gradient mt-1">
              that perform.
            </span>
          </h1>
        </motion.div>

        {/* Subtitle */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.8, delay: 0.25, ease: [0.25, 1, 0.5, 1] }}
          className="mb-10 max-w-2xl"
        >
          <p
            className="text-lg md:text-2xl leading-relaxed font-light"
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
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.8, delay: 0.4, ease: [0.25, 1, 0.5, 1] }}
          className="flex flex-wrap items-center gap-5 mb-24 z-20 relative"
        >
          <motion.div whileHover={{ scale: 1.02 }} whileTap={{ scale: 0.98 }}>
            <Link href="#projects" className="btn-primary" aria-label="View My Work">
              View My Work
              <ArrowRight className="w-5 h-5 group-hover:translate-x-1 duration-300 transition-transform" />
            </Link>
          </motion.div>
          <motion.div whileHover={{ scale: 1.02 }} whileTap={{ scale: 0.98 }}>
            <Link href="#contact" className="btn-outline" aria-label="Hire Me">
              Hire Me
            </Link>
          </motion.div>
        </motion.div>

        {/* Stats + Location */}
        <motion.div
          style={{ y: y2 }}
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.8, delay: 0.55, ease: [0.25, 1, 0.5, 1] }}
          className="flex flex-col md:flex-row md:items-center gap-8 md:gap-14 pt-10"
          style={{ borderTop: '1px solid var(--border)' }}
        >
          <div className="flex flex-wrap gap-8 md:gap-14">
            {stats.map((stat, i) => (
              <motion.div 
                key={stat.label} 
                className="flex flex-col gap-1.5"
                initial={{ opacity: 0, y: 10 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.5, delay: 0.6 + (i * 0.1) }}
              >
                <span className="text-4xl md:text-5xl font-display font-black text-gradient-static tracking-tight">
                  {stat.value}
                </span>
                <span
                  className="text-xs md:text-sm uppercase tracking-widest font-bold"
                  style={{ color: 'var(--text-tertiary)' }}
                >
                  {stat.label}
                </span>
              </motion.div>
            ))}
          </div>
          
          <motion.div
            initial={{ opacity: 0, x: -10 }}
            animate={{ opacity: 1, x: 0 }}
            transition={{ duration: 0.5, delay: 0.9 }}
            className="flex items-center gap-2.5 md:ml-auto mt-4 md:mt-0 glass px-5 py-3 rounded-2xl"
            style={{ color: 'var(--text-secondary)' }}
          >
            <div className="flex bg-accent/10 p-2 rounded-full text-accent dark:text-accent-hover">
              <MapPin className="w-4 h-4" />
            </div>
            <span className="text-sm font-semibold tracking-wide">Cairo, Egypt</span>
          </motion.div>
        </motion.div>
      </motion.div>

      {/* Scroll Indicator */}
      <motion.div
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ delay: 1.2, duration: 1 }}
        className="absolute bottom-8 left-1/2 -translate-x-1/2 flex flex-col items-center gap-3 z-0"
      >
        <span className="text-[10px] uppercase tracking-[0.2em] font-bold" style={{ color: 'var(--text-tertiary)' }}>
          Scroll Discover
        </span>
        <div className="w-[1px] h-12 relative overflow-hidden bg-black/10 dark:bg-white/10">
          <motion.div
            animate={{ y: ['-100%', '200%'] }}
            transition={{ duration: 2, repeat: Infinity, ease: 'linear' }}
            className="absolute top-0 left-0 w-full h-1/2"
            style={{ background: 'linear-gradient(to bottom, transparent, var(--accent), transparent)' }}
          />
        </div>
      </motion.div>
    </section>
  );
}

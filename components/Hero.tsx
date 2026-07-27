'use client';

import { motion, useScroll, useTransform, AnimatePresence } from 'framer-motion';
import Link from 'next/link';
import { ArrowRight, MapPin } from 'lucide-react';
import { useRef, useState, useEffect } from 'react';

const stats = [
  { value: '2+', label: 'Years Experience' },
  { value: '10+', label: 'Projects Shipped' },
  { value: '30%', label: 'Faster Workflows' },
];

const cyclingWords = ['interfaces', 'experiences', 'products', 'systems'];

const codeLines = [
  { tokens: [{ t: '// Building things people love', c: '#6a9955' }] },
  { tokens: [{ t: 'const ', c: 'var(--accent)' }, { t: 'engineer', c: '#9cdcfe' }, { t: ' = {', c: 'var(--text-primary)' }] },
  { tokens: [{ t: '  name', c: '#9cdcfe' }, { t: ': ', c: 'var(--text-primary)' }, { t: '"Mohamed Hussein"', c: '#ce9178' }, { t: ',', c: 'var(--text-secondary)' }] },
  { tokens: [{ t: '  role', c: '#9cdcfe' }, { t: ': ', c: 'var(--text-primary)' }, { t: '"Frontend Engineer"', c: '#ce9178' }, { t: ',', c: 'var(--text-secondary)' }] },
  { tokens: [{ t: '  stack', c: '#9cdcfe' }, { t: ': ', c: 'var(--text-primary)' }, { t: '["React", "Next.js"]', c: '#ce9178' }, { t: ',', c: 'var(--text-secondary)' }] },
  { tokens: [{ t: '  available', c: '#9cdcfe' }, { t: ': ', c: 'var(--text-primary)' }, { t: 'true', c: '#569cd6' }, { t: ',', c: 'var(--text-secondary)' }] },
  { tokens: [{ t: '};', c: 'var(--accent)' }] },
  { tokens: [{ t: '', c: '' }] },
  { tokens: [{ t: 'engineer', c: '#dcdcaa' }, { t: '.', c: 'var(--text-primary)' }, { t: 'build', c: '#dcdcaa' }, { t: '(', c: 'var(--text-primary)' }, { t: '"your next product"', c: '#ce9178' }, { t: ');', c: 'var(--text-primary)' }] },
];

function TerminalCard() {
  return (
    <div className="terminal-window relative">
      {/* Ambient glow behind card */}
      <div
        className="absolute -inset-6 -z-10 rounded-3xl blur-3xl opacity-30"
        style={{ background: 'linear-gradient(135deg, var(--gradient-from), var(--gradient-to))' }}
      />

      {/* Window chrome */}
      <div
        className="flex items-center gap-2 px-5 py-3.5 border-b"
        style={{ borderColor: 'var(--border)', background: 'var(--surface-elevated)' }}
      >
        <span className="w-3 h-3 rounded-full bg-red-400/80" />
        <span className="w-3 h-3 rounded-full bg-yellow-400/80" />
        <span className="w-3 h-3 rounded-full bg-green-400/80" />
        <span className="ml-4 text-xs font-mono" style={{ color: 'var(--text-tertiary)' }}>
          engineer.ts
        </span>
        <span
          className="ml-auto text-[10px] font-bold uppercase tracking-widest px-2 py-0.5 rounded-full"
          style={{ background: 'var(--badge-bg)', color: 'var(--badge-text)', border: '1px solid var(--badge-border)' }}
        >
          TypeScript
        </span>
      </div>

      {/* Code body */}
      <div className="p-6 font-mono text-sm leading-[1.9]" style={{ background: 'var(--surface-elevated)' }}>
        {codeLines.map((line, i) => (
          <motion.div
            key={i}
            initial={{ opacity: 0, x: -8 }}
            animate={{ opacity: 1, x: 0 }}
            transition={{ delay: 0.9 + i * 0.07, duration: 0.3, ease: 'easeOut' }}
            className="flex items-start gap-3"
          >
            <span
              className="select-none w-5 text-right shrink-0 text-[0.65rem] leading-[1.9]"
              style={{ color: 'var(--text-tertiary)', opacity: 0.35 }}
            >
              {i + 1}
            </span>
            <span>
              {line.tokens.map((tok, j) => (
                <span key={j} style={{ color: tok.c }}>
                  {tok.t}
                </span>
              ))}
            </span>
          </motion.div>
        ))}
        {/* Blinking cursor */}
        <motion.span
          animate={{ opacity: [1, 0, 1] }}
          transition={{ duration: 1.1, repeat: Infinity, ease: 'linear' }}
          className="inline-block w-[9px] h-[18px] rounded-sm ml-1 align-middle"
          style={{ background: 'var(--accent)' }}
        />
      </div>
    </div>
  );
}

export default function Hero() {
  const containerRef = useRef<HTMLDivElement>(null);
  const [wordIndex, setWordIndex] = useState(0);

  const { scrollYProgress } = useScroll({
    target: containerRef,
    offset: ['start start', 'end start'],
  });

  const y1 = useTransform(scrollYProgress, [0, 1], [0, 140]);
  const y2 = useTransform(scrollYProgress, [0, 1], [0, 70]);
  const opacity = useTransform(scrollYProgress, [0, 0.75], [1, 0]);

  useEffect(() => {
    const id = setInterval(() => {
      setWordIndex((prev) => (prev + 1) % cyclingWords.length);
    }, 2600);
    return () => clearInterval(id);
  }, []);

  return (
    <section
      id="hero"
      ref={containerRef}
      className="min-h-[100svh] flex flex-col justify-center relative px-6 md:px-8 pt-24 pb-12 overflow-hidden"
    >
      <motion.div
        style={{ y: y1, opacity }}
        className="max-w-6xl mx-auto w-full relative z-10 grid grid-cols-1 lg:grid-cols-[1fr_480px] gap-16 items-center"
      >
        {/* ── Left Content ── */}
        <div>
          {/* Status badge */}
          <motion.div
            initial={{ opacity: 0, scale: 0.92, y: -8 }}
            animate={{ opacity: 1, scale: 1, y: 0 }}
            transition={{ duration: 0.6, ease: [0.25, 1, 0.5, 1] }}
            className="mb-8 inline-block"
          >
            <div
              className="inline-flex items-center gap-3 px-4 py-2.5 rounded-full text-sm font-semibold backdrop-blur-md hover:scale-105 transition-transform duration-300 cursor-default"
              style={{
                background: 'var(--badge-bg)',
                border: '1px solid var(--badge-border)',
                color: 'var(--badge-text)',
                boxShadow: '0 4px 16px var(--accent-light)',
              }}
            >
              <span className="status-dot" />
              <span className="tracking-tight">Available for new opportunities</span>
            </div>
          </motion.div>

          {/* Headline with cycling word */}
          <motion.div
            initial={{ opacity: 0, y: 28 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.8, delay: 0.12, ease: [0.25, 1, 0.5, 1] }}
            className="mb-6"
          >
            <h1 className="font-display leading-[1.04] tracking-tighter text-[clamp(2.8rem,7vw,6rem)]">
              <span className="block" style={{ color: 'var(--text-primary)' }}>
                I build
              </span>
              {/* Cycling word — overflow hidden clips exit animation */}
              <span
                className="block text-gradient overflow-hidden"
                style={{ height: '1.1em' }}
              >
                <AnimatePresence mode="wait" initial={false}>
                  <motion.span
                    key={wordIndex}
                    initial={{ y: '110%', opacity: 0 }}
                    animate={{ y: '0%', opacity: 1 }}
                    exit={{ y: '-110%', opacity: 0 }}
                    transition={{ duration: 0.48, ease: [0.25, 1, 0.5, 1] }}
                    className="block"
                  >
                    {cyclingWords[wordIndex]}
                  </motion.span>
                </AnimatePresence>
              </span>
              <span className="block" style={{ color: 'var(--text-primary)' }}>
                that perform.
              </span>
            </h1>
          </motion.div>

          {/* Subtitle */}
          <motion.p
            initial={{ opacity: 0, y: 18 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.8, delay: 0.26, ease: [0.25, 1, 0.5, 1] }}
            className="mb-10 max-w-xl text-lg md:text-xl leading-relaxed font-light"
            style={{ color: 'var(--text-secondary)' }}
          >
            Frontend Engineer crafting{' '}
            <span style={{ color: 'var(--text-primary)', fontWeight: 600 }}>
              high-performance React applications
            </span>{' '}
            with pixel-perfect precision and scalable architecture.
          </motion.p>

          {/* CTA buttons */}
          <motion.div
            initial={{ opacity: 0, y: 16 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.8, delay: 0.4, ease: [0.25, 1, 0.5, 1] }}
            className="flex flex-wrap items-center gap-4 mb-16 relative z-20"
          >
            <motion.div whileHover={{ scale: 1.04 }} whileTap={{ scale: 0.96 }}>
              <Link href="#projects" className="btn-primary group" aria-label="View My Work">
                View My Work
                <ArrowRight className="w-5 h-5 transition-transform duration-300 group-hover:translate-x-1.5" />
              </Link>
            </motion.div>
            <motion.div whileHover={{ scale: 1.04 }} whileTap={{ scale: 0.96 }}>
              <Link href="#contact" className="btn-outline" aria-label="Hire Me">
                Hire Me
              </Link>
            </motion.div>
          </motion.div>

          {/* Stats row */}
          <motion.div
            initial={{ opacity: 0, y: 16 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.8, delay: 0.55, ease: [0.25, 1, 0.5, 1] }}
            className="flex flex-wrap items-end gap-10 pt-8"
            style={{ borderTop: '1px solid var(--border)' }}
          >
            {stats.map((stat, i) => (
              <motion.div
                key={stat.label}
                className="flex flex-col gap-1"
                initial={{ opacity: 0, y: 10 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.5, delay: 0.62 + i * 0.1 }}
              >
                <span className="text-3xl md:text-4xl font-display font-black text-gradient-static tracking-tight">
                  {stat.value}
                </span>
                <span
                  className="text-[0.65rem] uppercase tracking-[0.18em] font-bold"
                  style={{ color: 'var(--text-tertiary)' }}
                >
                  {stat.label}
                </span>
              </motion.div>
            ))}

            <motion.div
              initial={{ opacity: 0, x: 10 }}
              animate={{ opacity: 1, x: 0 }}
              transition={{ duration: 0.5, delay: 0.95 }}
              className="flex items-center gap-2.5 ml-auto glass px-4 py-2.5 rounded-2xl self-center"
              style={{ color: 'var(--text-secondary)' }}
            >
              <div className="bg-accent/10 p-1.5 rounded-full">
                <MapPin className="w-3.5 h-3.5" style={{ color: 'var(--accent)' }} />
              </div>
              <span className="text-sm font-semibold tracking-wide">Cairo, Egypt</span>
            </motion.div>
          </motion.div>
        </div>

        {/* ── Right: Terminal Card ── */}
        <motion.div
          initial={{ opacity: 0, scale: 0.88, y: 24 }}
          animate={{ opacity: 1, scale: 1, y: 0 }}
          transition={{ duration: 1.1, delay: 0.52, ease: [0.25, 1, 0.5, 1] }}
          style={{ y: y2 }}
          className="hidden lg:block"
        >
          <TerminalCard />
        </motion.div>
      </motion.div>

      {/* Scroll indicator */}
      <motion.div
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ delay: 1.6, duration: 1 }}
        className="absolute bottom-8 left-1/2 -translate-x-1/2 flex flex-col items-center gap-2.5 z-10"
      >
        <span
          className="text-[9px] uppercase tracking-[0.25em] font-bold"
          style={{ color: 'var(--text-tertiary)' }}
        >
          Scroll to Explore
        </span>
        <div
          className="w-px h-10 relative overflow-hidden rounded-full"
          style={{ background: 'var(--border)' }}
        >
          <motion.div
            animate={{ y: ['-100%', '200%'] }}
            transition={{ duration: 1.8, repeat: Infinity, ease: 'linear' }}
            className="absolute top-0 left-0 w-full h-1/2 rounded-full"
            style={{ background: 'linear-gradient(to bottom, transparent, var(--accent), transparent)' }}
          />
        </div>
      </motion.div>
    </section>
  );
}

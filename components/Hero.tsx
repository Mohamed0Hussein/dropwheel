'use client';

import dynamic from 'next/dynamic';
import { motion, useScroll, useTransform, AnimatePresence } from 'framer-motion';
import Link from 'next/link';
import { ArrowRight, MapPin, Star, Download, Github, Linkedin, Mail } from 'lucide-react';
import { useRef, useState, useEffect } from 'react';
import MagneticButton from './MagneticButton';

// Lazy-load the entire Three.js bundle — never sent to the client on SSR
// and never requested on mobile viewports (<640 px).
const Scene3D = dynamic(() => import('./Scene3D'), { ssr: false });

const stats = [
  { value: '2+', label: 'Years Experience' },
  { value: '10+', label: 'Projects Shipped' },
  { value: '30%', label: 'Faster Workflows' },
];

const cyclingWords = ['interfaces', 'experiences', 'products', 'systems'];

const heroSocials = [
  { name: 'GitHub', href: 'https://github.com/Mohamed0Hussein', icon: Github },
  { name: 'LinkedIn', href: 'https://www.linkedin.com/in/mohamed-hussein-0x1/', icon: Linkedin },
  { name: 'Email', href: 'mailto:mohamed.hussein0x1@gmail.com', icon: Mail },
];

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

/* Floating particles — generated client-side only (Math.random() at module
   scope would produce different values on server vs. client and trigger a
   hydration mismatch on every load). */
function generateParticles() {
  return Array.from({ length: 20 }, (_, i) => ({
    id: i,
    x: Math.random() * 100,
    y: Math.random() * 100,
    size: Math.random() * 3 + 1,
    delay: Math.random() * 8,
    duration: Math.random() * 12 + 10,
  }));
}

type ParticleData = ReturnType<typeof generateParticles>[number];

function Particle({ p }: { p: ParticleData }) {
  return (
    <motion.div
      className="absolute rounded-full pointer-events-none"
      style={{
        left: `${p.x}%`,
        top: `${p.y}%`,
        width: p.size,
        height: p.size,
        background: 'var(--accent)',
        opacity: 0.25,
        boxShadow: `0 0 ${p.size * 3}px var(--accent-glow)`,
      }}
      animate={{
        y: [0, -60, 0],
        x: [0, Math.random() * 30 - 15, 0],
        opacity: [0.1, 0.4, 0.1],
        scale: [0.8, 1.2, 0.8],
      }}
      transition={{
        duration: p.duration,
        delay: p.delay,
        repeat: Infinity,
        ease: 'easeInOut',
      }}
    />
  );
}

function TerminalCard() {
  return (
    <div className="terminal-window relative group">
      {/* Multi-layer ambient glow */}
      <div
        className="absolute -inset-8 -z-10 rounded-3xl opacity-40 blur-3xl transition-opacity duration-500 group-hover:opacity-60"
        style={{ background: 'radial-gradient(ellipse at center, var(--orb-1) 0%, var(--orb-2) 50%, transparent 70%)' }}
      />
      <div
        className="absolute -inset-4 -z-10 rounded-3xl opacity-20 blur-xl"
        style={{ background: 'linear-gradient(135deg, var(--gradient-from), var(--gradient-to))' }}
      />

      {/* Window chrome */}
      <div
        className="flex items-center gap-2 px-5 py-4 border-b"
        style={{ borderColor: 'var(--border-strong)', background: 'var(--surface-elevated)' }}
      >
        <span className="w-3 h-3 rounded-full" style={{ background: '#ff5f57' }} />
        <span className="w-3 h-3 rounded-full" style={{ background: '#febc2e' }} />
        <span className="w-3 h-3 rounded-full" style={{ background: '#28c840' }} />
        <span className="ml-4 text-xs font-mono" style={{ color: 'var(--text-tertiary)', fontFamily: 'JetBrains Mono, monospace' }}>
          engineer.ts
        </span>
        <span
          className="ml-auto text-[10px] font-bold uppercase tracking-widest px-2.5 py-1 rounded-full"
          style={{ background: 'var(--badge-bg)', color: 'var(--badge-text)', border: '1px solid var(--badge-border)' }}
        >
          TypeScript
        </span>
      </div>

      {/* Line numbers column */}
      <div className="flex" style={{ background: 'var(--surface-elevated)' }}>
        <div
          className="flex flex-col py-6 px-3 select-none border-r"
          style={{ borderColor: 'var(--border)', background: 'rgba(0,0,0,0.04)' }}
        >
          {codeLines.map((_, i) => (
            <span
              key={i}
              className="text-[0.65rem] leading-[1.9] text-right w-4"
              style={{ color: 'var(--text-tertiary)', opacity: 0.4, fontFamily: 'JetBrains Mono, monospace' }}
            >
              {i + 1}
            </span>
          ))}
        </div>

        {/* Code body */}
        <div className="p-6 flex-1 overflow-x-auto">
          {codeLines.map((line, i) => (
            <motion.div
              key={i}
              initial={{ opacity: 0, x: -12 }}
              animate={{ opacity: 1, x: 0 }}
              transition={{ delay: 0.9 + i * 0.07, duration: 0.35, ease: 'easeOut' }}
              className="flex items-start text-sm leading-[1.9] whitespace-nowrap"
              style={{ fontFamily: 'JetBrains Mono, monospace' }}
            >
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

      {/* Bottom status bar */}
      <div
        className="flex items-center justify-between px-5 py-2.5 border-t"
        style={{
          borderColor: 'var(--border)',
          background: 'linear-gradient(to right, var(--accent-light), transparent)',
        }}
      >
        <span className="text-[0.6rem] font-mono font-semibold tracking-wider" style={{ color: 'var(--text-tertiary)' }}>
          Ln 9, Col 38
        </span>
        <div className="flex items-center gap-3">
          <span className="text-[0.6rem] font-mono" style={{ color: 'var(--text-tertiary)' }}>UTF-8</span>
          <span
            className="text-[0.6rem] font-bold uppercase px-2 py-0.5 rounded"
            style={{ background: 'var(--accent-light)', color: 'var(--accent)' }}
          >
            TypeScript
          </span>
        </div>
      </div>
    </div>
  );
}

export default function Hero() {
  const containerRef = useRef<HTMLDivElement>(null);
  const [wordIndex, setWordIndex] = useState(0);
  // Only mount the WebGL canvas on sm+ viewports
  const [showScene, setShowScene] = useState(false);
  // Generated client-side only post-mount — see generateParticles() note above
  const [particles, setParticles] = useState<ParticleData[]>([]);

  useEffect(() => {
    // Math.random() must stay off the server-render path (see note above),
    // so this one-time client-only init genuinely needs an effect.
    // eslint-disable-next-line react-hooks/set-state-in-effect
    setParticles(generateParticles());
  }, []);

  const { scrollYProgress } = useScroll({
    target: containerRef,
    offset: ['start start', 'end start'],
  });

  const y1 = useTransform(scrollYProgress, [0, 1], [0, 160]);
  const y2 = useTransform(scrollYProgress, [0, 1], [0, 80]);
  const opacity = useTransform(scrollYProgress, [0, 0.7], [1, 0]);
  const scale = useTransform(scrollYProgress, [0, 0.7], [1, 0.97]);

  useEffect(() => {
    const id = setInterval(() => {
      setWordIndex((prev) => (prev + 1) % cyclingWords.length);
    }, 2600);
    return () => clearInterval(id);
  }, []);

  // Media-query guard: create / destroy the GL context based on viewport width
  useEffect(() => {
    const mq = window.matchMedia('(min-width: 640px)');
    setShowScene(mq.matches);
    const handler = (e: MediaQueryListEvent) => setShowScene(e.matches);
    mq.addEventListener('change', handler);
    return () => mq.removeEventListener('change', handler);
  }, []);

  return (
    <section
      id="hero"
      ref={containerRef}
      className="min-h-[100svh] flex flex-col justify-center relative px-6 md:px-8 pt-28 pb-16 overflow-hidden"
    >
      {/* ── 3-D Background Scene (sm+, lazy-loaded) ── */}
      {showScene && (
        <div
          className="absolute inset-0 z-0 pointer-events-none"
          aria-hidden="true"
        >
          <Scene3D />
        </div>
      )}

      {/* Floating particles */}
      <div className="absolute inset-0 pointer-events-none overflow-hidden">
        {particles.map((p) => (
          <Particle key={p.id} p={p} />
        ))}
      </div>

      {/* Big decorative orb — top right */}
      <motion.div
        className="absolute top-[-20%] right-[-10%] w-[700px] h-[700px] rounded-full pointer-events-none"
        style={{
          background: 'radial-gradient(circle at 40% 40%, var(--orb-1) 0%, var(--orb-2) 40%, transparent 70%)',
          filter: 'blur(80px)',
        }}
        animate={{ scale: [1, 1.08, 1], rotate: [0, 5, 0] }}
        transition={{ duration: 20, repeat: Infinity, ease: 'easeInOut' }}
      />

      {/* Second orb — bottom left */}
      <motion.div
        className="absolute bottom-[-15%] left-[-10%] w-[500px] h-[500px] rounded-full pointer-events-none"
        style={{
          background: 'radial-gradient(circle at 60% 60%, var(--orb-3) 0%, transparent 70%)',
          filter: 'blur(80px)',
        }}
        animate={{ scale: [1, 1.12, 1], rotate: [0, -6, 0] }}
        transition={{ duration: 26, repeat: Infinity, ease: 'easeInOut', delay: 4 }}
      />

      {/* Geometric decorations */}
      <motion.div
        className="absolute top-1/4 right-1/3 w-24 h-24 rounded-2xl rotate-12 opacity-[0.04] pointer-events-none"
        style={{ border: '2px solid var(--accent)', background: 'transparent' }}
        animate={{ rotate: [12, 24, 12], scale: [1, 1.1, 1] }}
        transition={{ duration: 16, repeat: Infinity, ease: 'easeInOut' }}
      />
      <motion.div
        className="absolute bottom-1/3 right-1/4 w-16 h-16 rounded-xl -rotate-6 opacity-[0.06] pointer-events-none"
        style={{ background: 'var(--accent-2)', filter: 'blur(2px)' }}
        animate={{ rotate: [-6, -18, -6], y: [0, -20, 0] }}
        transition={{ duration: 12, repeat: Infinity, ease: 'easeInOut', delay: 2 }}
      />

      {/* Star decorations */}
      {[
        { top: '15%', left: '8%', delay: 0 },
        { top: '60%', left: '5%', delay: 1.5 },
        { top: '30%', right: '6%', delay: 2.5 },
        { top: '75%', right: '15%', delay: 0.8 },
      ].map((pos, i) => (
        <motion.div
          key={i}
          className="absolute pointer-events-none hidden lg:block"
          style={{ ...pos }}
          animate={{ opacity: [0.15, 0.5, 0.15], scale: [0.8, 1.2, 0.8] }}
          transition={{ duration: 3 + i, repeat: Infinity, delay: pos.delay, ease: 'easeInOut' }}
        >
          <Star
            className="w-3 h-3"
            style={{ color: 'var(--accent)', fill: 'var(--accent)' }}
          />
        </motion.div>
      ))}

      <motion.div
        style={{ y: y1, opacity, scale }}
        className="max-w-6xl mx-auto w-full relative z-10 grid grid-cols-1 lg:grid-cols-[1fr_500px] gap-16 items-center"
      >
        {/* ── Left Content ── */}
        <div>
          {/* Status badge */}
          <motion.div
            initial={{ opacity: 0, scale: 0.88, y: -12 }}
            animate={{ opacity: 1, scale: 1, y: 0 }}
            transition={{ duration: 0.7, ease: [0.25, 1, 0.5, 1] }}
            className="mb-9 inline-block"
          >
            <motion.div
              whileHover={{ scale: 1.05 }}
              className="inline-flex items-center gap-3 px-5 py-2.5 rounded-full text-sm font-semibold backdrop-blur-md cursor-default"
              style={{
                background: 'var(--badge-bg)',
                border: '1px solid var(--badge-border)',
                color: 'var(--badge-text)',
                boxShadow: '0 4px 20px var(--accent-glow), inset 0 1px 0 rgba(255,255,255,0.15)',
              }}
            >
              <span className="status-dot" />
              <span className="tracking-tight">Available for new opportunities</span>
            </motion.div>
          </motion.div>

          {/* Headline with cycling word */}
          <motion.div
            initial={{ opacity: 0, y: 32 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.9, delay: 0.15, ease: [0.25, 1, 0.5, 1] }}
            className="mb-7"
          >
            <h1
              className="font-display leading-[1.04] tracking-tighter"
              style={{ fontSize: 'clamp(3rem, 7.5vw, 6.5rem)' }}
            >
              <span
                className="block"
                style={{ color: 'var(--text-primary)', letterSpacing: '-0.04em' }}
              >
                I build
              </span>

              {/* Cycling word */}
              <span
                className="block overflow-hidden"
                style={{ height: '1.08em', letterSpacing: '-0.04em' }}
              >
                <AnimatePresence mode="wait" initial={false}>
                  <motion.span
                    key={wordIndex}
                    initial={{ y: '110%', opacity: 0, rotateX: -15 }}
                    animate={{ y: '0%', opacity: 1, rotateX: 0 }}
                    exit={{ y: '-110%', opacity: 0, rotateX: 15 }}
                    transition={{ duration: 0.5, ease: [0.25, 1, 0.5, 1] }}
                    className="block text-gradient"
                  >
                    {cyclingWords[wordIndex]}
                  </motion.span>
                </AnimatePresence>
              </span>

              <span
                className="block"
                style={{ color: 'var(--text-primary)', letterSpacing: '-0.04em' }}
              >
                that{' '}
                <span className="relative inline-block">
                  perform
                  {/* Underline accent */}
                  <motion.span
                    className="absolute -bottom-1 left-0 right-0 h-1 rounded-full"
                    style={{ background: 'linear-gradient(to right, var(--gradient-from), var(--gradient-to))' }}
                    initial={{ scaleX: 0, originX: 0 }}
                    animate={{ scaleX: 1 }}
                    transition={{ delay: 1.2, duration: 0.8, ease: [0.25, 1, 0.5, 1] }}
                  />
                </span>
                .
              </span>
            </h1>
          </motion.div>

          {/* Subtitle */}
          <motion.p
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.9, delay: 0.3, ease: [0.25, 1, 0.5, 1] }}
            className="mb-10 max-w-xl text-lg md:text-xl leading-relaxed"
            style={{ color: 'var(--text-secondary)', fontWeight: 400 }}
          >
            Frontend Engineer crafting{' '}
            <span style={{ color: 'var(--text-primary)', fontWeight: 600 }}>
              high-performance React applications
            </span>{' '}
            with pixel-perfect precision and scalable architecture.
          </motion.p>

          {/* CTA buttons */}
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.9, delay: 0.44, ease: [0.25, 1, 0.5, 1] }}
            className="flex flex-wrap items-center gap-4 mb-8 relative z-20"
          >
            <MagneticButton>
              <motion.div whileHover={{ scale: 1.04 }} whileTap={{ scale: 0.96 }}>
                <Link href="#projects" className="btn-primary group" aria-label="View My Work">
                  View My Work
                  <ArrowRight className="w-5 h-5 transition-transform duration-300 group-hover:translate-x-1.5" />
                </Link>
              </motion.div>
            </MagneticButton>
            <motion.div whileHover={{ scale: 1.04 }} whileTap={{ scale: 0.96 }}>
              <Link href="#contact" className="btn-outline" aria-label="Hire Me">
                Hire Me
              </Link>
            </motion.div>
            <motion.div whileHover={{ scale: 1.04 }} whileTap={{ scale: 0.96 }}>
              <a
                href="/resume.pdf"
                download
                className="btn-ghost"
                style={{ border: '1px solid var(--border-strong)' }}
                aria-label="Download résumé (PDF)"
              >
                <Download className="w-4 h-4" />
                Resume
              </a>
            </motion.div>
          </motion.div>

          {/* Social links */}
          <motion.div
            initial={{ opacity: 0, y: 16 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.9, delay: 0.5, ease: [0.25, 1, 0.5, 1] }}
            className="flex items-center gap-3 mb-16 relative z-20"
          >
            {heroSocials.map((social) => (
              <motion.a
                key={social.name}
                href={social.href}
                target={social.name === 'Email' ? undefined : '_blank'}
                rel={social.name === 'Email' ? undefined : 'noopener noreferrer'}
                aria-label={social.name}
                className="flex items-center justify-center w-10 h-10 rounded-xl"
                style={{
                  background: 'var(--surface-elevated)',
                  border: '1px solid var(--border-strong)',
                  color: 'var(--text-tertiary)',
                }}
                whileHover={{ scale: 1.12, y: -2, color: 'var(--accent)' }}
                whileTap={{ scale: 0.92 }}
              >
                <social.icon className="w-4 h-4" />
              </motion.a>
            ))}
          </motion.div>

          {/* Stats row */}
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.9, delay: 0.58, ease: [0.25, 1, 0.5, 1] }}
            className="flex flex-wrap items-end gap-10 pt-8"
            style={{ borderTop: '1px solid var(--border)' }}
          >
            {stats.map((stat, i) => (
              <motion.div
                key={stat.label}
                className="flex flex-col gap-1"
                initial={{ opacity: 0, y: 12 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.55, delay: 0.65 + i * 0.12 }}
              >
                <span className="text-3xl md:text-4xl font-display font-black text-gradient-static tracking-tight">
                  {stat.value}
                </span>
                <span
                  className="text-[0.65rem] uppercase tracking-[0.2em] font-bold"
                  style={{ color: 'var(--text-tertiary)' }}
                >
                  {stat.label}
                </span>
              </motion.div>
            ))}

            {/* Location badge */}
            <motion.div
              initial={{ opacity: 0, x: 12 }}
              animate={{ opacity: 1, x: 0 }}
              transition={{ duration: 0.55, delay: 1.0 }}
              className="flex items-center gap-2.5 ml-auto glass px-4 py-2.5 rounded-2xl self-center"
              style={{ color: 'var(--text-secondary)' }}
            >
              <div
                className="p-1.5 rounded-full"
                style={{ background: 'var(--accent-light)' }}
              >
                <MapPin className="w-3.5 h-3.5" style={{ color: 'var(--accent)' }} />
              </div>
              <span className="text-sm font-semibold tracking-wide">Cairo, Egypt</span>
            </motion.div>
          </motion.div>
        </div>

        {/* ── Right: Terminal Card ── */}
        <motion.div
          initial={{ opacity: 0, scale: 0.85, y: 32 }}
          animate={{ opacity: 1, scale: 1, y: 0 }}
          transition={{ duration: 1.2, delay: 0.56, ease: [0.25, 1, 0.5, 1] }}
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
        transition={{ delay: 1.8, duration: 1 }}
        className="absolute bottom-8 left-1/2 -translate-x-1/2 flex flex-col items-center gap-2.5 z-10"
      >
        <span
          className="text-[9px] uppercase tracking-[0.28em] font-bold"
          style={{ color: 'var(--text-tertiary)' }}
        >
          Scroll
        </span>
        <div
          className="w-px h-10 relative overflow-hidden rounded-full"
          style={{ background: 'var(--border-strong)' }}
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

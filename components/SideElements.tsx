'use client';

import { motion, useScroll, useSpring, useTransform } from 'framer-motion';
import { Github, Linkedin } from 'lucide-react';

const sideLinks = [
  { href: 'https://github.com/Mohamed0Hussein',                icon: Github,   label: 'GitHub'   },
  { href: 'https://www.linkedin.com/in/mohamed-hussein-0x1/', icon: Linkedin, label: 'LinkedIn' },
];

export default function SideElements() {
  const { scrollYProgress } = useScroll();

  const scaleY = useSpring(scrollYProgress, {
    stiffness: 100,
    damping: 30,
    restDelta: 0.001,
  });

  const emailOpacity = useTransform(scrollYProgress, [0.9, 1], [1, 0]);

  return (
    <>
      {/* ── Left sidebar: scroll progress + email ── */}
      <motion.div
        initial={{ opacity: 0, x: -20 }}
        animate={{ opacity: 1, x: 0 }}
        transition={{ delay: 1.4, duration: 0.8, ease: [0.25, 1, 0.5, 1] }}
        className="fixed left-5 lg:left-8 top-1/2 -translate-y-1/2 z-40 hidden xl:flex flex-col items-center gap-5"
      >
        {/* Social icons */}
        {sideLinks.map((link) => (
          <motion.a
            key={link.label}
            href={link.href}
            target="_blank"
            rel="noopener noreferrer"
            aria-label={link.label}
            className="flex items-center justify-center w-8 h-8 rounded-lg transition-all duration-300"
            style={{
              color: 'var(--text-tertiary)',
              border: '1px solid var(--border)',
              background: 'var(--surface-elevated)',
            }}
            whileHover={{ scale: 1.15, y: -2, color: 'var(--accent)' }}
            whileTap={{ scale: 0.9 }}
          >
            <link.icon className="w-3.5 h-3.5" />
          </motion.a>
        ))}

        {/* Divider */}
        <div className="w-px h-6" style={{ background: 'var(--border-strong)' }} />

        {/* Scroll progress bar */}
        <div
          className="h-24 w-[2px] relative overflow-hidden rounded-full"
          style={{ background: 'var(--border)' }}
        >
          <motion.div
            className="absolute top-0 left-0 w-full rounded-full origin-top"
            style={{
              scaleY,
              background: 'linear-gradient(180deg, var(--gradient-from), var(--gradient-to))',
              height: '100%',
            }}
          />
        </div>

        {/* Status dot */}
        <div className="flex items-center">
          <span className="status-dot" style={{ width: '7px', height: '7px' }} />
        </div>

        {/* Vertical email */}
        <motion.a
          href="mailto:mohamed.hussein0x1@gmail.com"
          aria-label="Email me"
          className="vertical-text text-[0.6rem] tracking-[0.3em] font-semibold transition-colors duration-300 hover:text-accent"
          style={{ opacity: emailOpacity, color: 'var(--text-tertiary)' }}
          whileHover={{ color: 'var(--accent)' }}
        >
          mohamed.hussein0x1@gmail.com
        </motion.a>
      </motion.div>

      {/* ── Right sidebar: scroll % indicator ── */}
      <motion.div
        initial={{ opacity: 0, x: 20 }}
        animate={{ opacity: 1, x: 0 }}
        transition={{ delay: 1.6, duration: 0.8, ease: [0.25, 1, 0.5, 1] }}
        className="fixed right-5 lg:right-8 top-1/2 -translate-y-1/2 z-40 hidden xl:flex flex-col items-center gap-3"
      >
        <motion.span
          className="vertical-text text-[0.6rem] tracking-[0.25em] font-bold"
          style={{ color: 'var(--text-tertiary)' }}
        >
          scroll
        </motion.span>
        <div
          className="h-16 w-[1px] rounded-full overflow-hidden"
          style={{ background: 'var(--border)' }}
        >
          <motion.div
            className="w-full rounded-full origin-top"
            style={{
              scaleY,
              background: 'var(--accent)',
              height: '100%',
            }}
          />
        </div>
      </motion.div>
    </>
  );
}

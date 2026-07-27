'use client';

import { motion } from 'framer-motion';
import Link from 'next/link';
import { Github, Linkedin, Mail, Heart } from 'lucide-react';

const links = [
  { label: 'GitHub',   href: 'https://github.com/Mohamed0Hussein',                icon: Github,   external: true  },
  { label: 'LinkedIn', href: 'https://www.linkedin.com/in/mohamed-hussein-0x1/', icon: Linkedin, external: true  },
  { label: 'Email',    href: 'mailto:mohamed.hussein0x1@gmail.com',               icon: Mail,     external: false },
];

export default function Footer() {
  const year = new Date().getFullYear();

  return (
    <footer className="relative z-10 px-6 md:px-8 pb-10 pt-6">
      {/* Top divider */}
      <div className="max-w-6xl mx-auto">
        <div className="section-divider mb-8" />

        <motion.div
          initial={{ opacity: 0, y: 16 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.6, ease: [0.25, 1, 0.5, 1] }}
          className="flex flex-col sm:flex-row items-center justify-between gap-6"
        >
          {/* Left: Brand + year */}
          <div className="flex flex-col sm:flex-row items-center gap-2 sm:gap-4">
            <span
              className="text-sm font-display font-bold tracking-tight"
              style={{ color: 'var(--text-primary)' }}
            >
              Mohamed <span className="text-gradient">Hussein</span>
            </span>
            <span className="hidden sm:block w-px h-4" style={{ background: 'var(--border-strong)' }} />
            <p className="text-xs flex items-center gap-1.5" style={{ color: 'var(--text-tertiary)' }}>
              © {year} · Crafted with
              <Heart className="w-3 h-3 text-red-400 fill-red-400" aria-hidden />
              &amp; Next.js
            </p>
          </div>

          {/* Right: Social icon links */}
          <div className="flex items-center gap-2">
            {links.map((link) => (
              <motion.div key={link.label} whileHover={{ scale: 1.12, y: -2 }} whileTap={{ scale: 0.92 }}>
                <Link
                  href={link.href}
                  target={link.external ? '_blank' : undefined}
                  rel={link.external ? 'noopener noreferrer' : undefined}
                  aria-label={link.label}
                  className="flex items-center justify-center w-9 h-9 rounded-xl transition-all duration-300 hover:border-accent"
                  style={{
                    background: 'var(--surface-elevated)',
                    border: '1px solid var(--border-strong)',
                    color: 'var(--text-tertiary)',
                  }}
                >
                  <link.icon className="w-4 h-4" />
                </Link>
              </motion.div>
            ))}
          </div>
        </motion.div>
      </div>
    </footer>
  );
}

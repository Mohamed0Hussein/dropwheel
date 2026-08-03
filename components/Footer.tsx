'use client';

import { motion } from 'framer-motion';
import Link from 'next/link';
import { Github, Linkedin, Mail, Heart, ArrowUpRight } from 'lucide-react';

const links = [
  { label: 'GitHub',   href: 'https://github.com/Mohamed0Hussein',                icon: Github,   external: true  },
  { label: 'LinkedIn', href: 'https://www.linkedin.com/in/mohamed-hussein-0x1/', icon: Linkedin, external: true  },
  { label: 'Email',    href: 'mailto:mohamed.hussein0x1@gmail.com',               icon: Mail,     external: false },
];

const navLinks = [
  { label: 'About',      href: '#about'      },
  { label: 'Projects',   href: '#projects'   },
  { label: 'Stack',      href: '#stack'      },
  { label: 'Experience', href: '#experience' },
  { label: 'Contact',    href: '#contact'    },
];

export default function Footer() {
  const year = new Date().getFullYear();

  return (
    <footer className="relative z-10 px-6 md:px-8 pb-10 pt-0">
      {/* Gradient top border */}
      <div
        className="h-px mb-12 w-full"
        style={{
          background: 'linear-gradient(to right, transparent, var(--gradient-from) 20%, var(--gradient-via) 50%, var(--gradient-to) 80%, transparent)',
          opacity: 0.4,
        }}
      />

      <div className="max-w-6xl mx-auto">
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.7, ease: [0.25, 1, 0.5, 1] }}
          className="flex flex-col gap-8"
        >
          {/* Top row: brand + nav */}
          <div className="flex flex-col md:flex-row items-start md:items-center justify-between gap-6">
            {/* Brand */}
            <Link href="/" className="group outline-none" aria-label="Home">
              <span
                className="text-xl font-display font-bold tracking-tight"
                style={{ color: 'var(--text-primary)', letterSpacing: '-0.04em' }}
              >
                Mohamed{' '}
                <span className="text-gradient">Hussein</span>
              </span>
              <p className="text-xs mt-1" style={{ color: 'var(--text-tertiary)' }}>
                Frontend Engineer · Cairo, Egypt
              </p>
            </Link>

            {/* Nav links */}
            <nav className="flex flex-wrap gap-x-6 gap-y-2" aria-label="Footer navigation">
              {navLinks.map((link) => (
                <Link
                  key={link.label}
                  href={link.href}
                  className="text-sm font-medium transition-colors duration-300 hover:text-accent"
                  style={{ color: 'var(--text-tertiary)' }}
                >
                  {link.label}
                </Link>
              ))}
            </nav>
          </div>

          {/* Bottom row: copy + social */}
          <div
            className="flex flex-col sm:flex-row items-center justify-between gap-4 pt-6"
            style={{ borderTop: '1px solid var(--border)' }}
          >
            {/* Copyright */}
            <p className="text-xs flex items-center gap-1.5" style={{ color: 'var(--text-tertiary)' }}>
              © {year} Mohamed Hussein · Crafted with
              <motion.span
                animate={{ scale: [1, 1.3, 1] }}
                transition={{ duration: 1.5, repeat: Infinity, ease: 'easeInOut' }}
                className="inline-block"
              >
                <Heart className="w-3.5 h-3.5 text-red-400 fill-red-400" aria-hidden />
              </motion.span>
              &amp; Next.js
            </p>

            {/* Social icon links */}
            <div className="flex items-center gap-2">
              {links.map((link) => (
                <motion.div key={link.label} whileHover={{ scale: 1.12, y: -2 }} whileTap={{ scale: 0.92 }}>
                  <Link
                    href={link.href}
                    target={link.external ? '_blank' : undefined}
                    rel={link.external ? 'noopener noreferrer' : undefined}
                    aria-label={link.label}
                    className="flex items-center justify-center w-9 h-9 rounded-xl transition-all duration-300 hover:border-accent hover:text-accent"
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

              {/* Back to top */}
              <motion.div whileHover={{ scale: 1.12, y: -2 }} whileTap={{ scale: 0.92 }}>
                <button
                  onClick={() => window.scrollTo({ top: 0, behavior: 'smooth' })}
                  aria-label="Back to top"
                  className="flex items-center justify-center w-9 h-9 rounded-xl transition-all duration-300 hover:border-accent hover:text-accent"
                  style={{
                    background: 'var(--accent-light)',
                    border: '1px solid var(--badge-border)',
                    color: 'var(--accent)',
                  }}
                >
                  <ArrowUpRight className="w-4 h-4 -rotate-45" />
                </button>
              </motion.div>
            </div>
          </div>
        </motion.div>
      </div>
    </footer>
  );
}

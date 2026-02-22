'use client';

import { motion } from 'framer-motion';
import Link from 'next/link';

export default function Footer() {
  const currentYear = new Date().getFullYear();

  return (
    <footer className="px-6 md:px-8 pb-10 relative z-10 pt-4">
      <div className="max-w-6xl mx-auto">
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.6, ease: [0.25, 1, 0.5, 1] }}
          className="flex flex-col sm:flex-row items-center justify-between gap-6 px-2"
        >
          <p
            className="text-sm font-medium"
            style={{ color: 'var(--text-tertiary)' }}
          >
            © {currentYear} Mohamed Hussein. Crafted with care & Next.js.
          </p>
          <div className="flex items-center gap-8">
            <Link
              href="https://github.com/Mohamed0Hussein"
              target="_blank"
              rel="noopener noreferrer"
              className="text-xs uppercase tracking-[0.15em] font-bold transition-all duration-300 hover:text-accent"
              style={{ color: 'var(--text-tertiary)' }}
            >
              GitHub
            </Link>
            <Link
              href="https://www.linkedin.com/in/mohamed-hussein-0x1/"
              target="_blank"
              rel="noopener noreferrer"
              className="text-xs uppercase tracking-[0.15em] font-bold transition-all duration-300 hover:text-accent"
              style={{ color: 'var(--text-tertiary)' }}
            >
              LinkedIn
            </Link>
            <Link
              href="mailto:mohamed.hussein0x1@gmail.com"
              className="text-xs uppercase tracking-[0.15em] font-bold transition-all duration-300 hover:text-accent"
              style={{ color: 'var(--text-tertiary)' }}
            >
              Email
            </Link>
          </div>
        </motion.div>
      </div>
    </footer>
  );
}

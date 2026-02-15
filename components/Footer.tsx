'use client';

import { motion } from 'framer-motion';
import Link from 'next/link';

export default function Footer() {
  const currentYear = new Date().getFullYear();

  return (
    <footer className="px-6 md:px-8 pb-8">
      <div className="max-w-6xl mx-auto">
        <div className="divider mb-8" />
        <motion.div
          initial={{ opacity: 0 }}
          whileInView={{ opacity: 1 }}
          viewport={{ once: true }}
          transition={{ duration: 0.5 }}
          className="flex flex-col sm:flex-row items-center justify-between gap-4"
        >
          <p
            className="text-sm"
            style={{ color: 'var(--text-tertiary)' }}
          >
            © {currentYear} Mohamed Hussein. Built with Next.js & Tailwind.
          </p>
          <div className="flex items-center gap-6">
            <Link
              href="https://github.com/Mohamed0Hussein"
              target="_blank"
              rel="noopener noreferrer"
              className="text-xs uppercase tracking-wider font-medium transition-colors duration-200"
              style={{ color: 'var(--text-tertiary)' }}
              onMouseEnter={(e) => (e.currentTarget.style.color = 'var(--text-primary)')}
              onMouseLeave={(e) => (e.currentTarget.style.color = 'var(--text-tertiary)')}
            >
              GitHub
            </Link>
            <Link
              href="https://www.linkedin.com/in/mohamed-hussein-0x1/"
              target="_blank"
              rel="noopener noreferrer"
              className="text-xs uppercase tracking-wider font-medium transition-colors duration-200"
              style={{ color: 'var(--text-tertiary)' }}
              onMouseEnter={(e) => (e.currentTarget.style.color = 'var(--text-primary)')}
              onMouseLeave={(e) => (e.currentTarget.style.color = 'var(--text-tertiary)')}
            >
              LinkedIn
            </Link>
            <Link
              href="mailto:mohamed.hussein0x1@gmail.com"
              className="text-xs uppercase tracking-wider font-medium transition-colors duration-200"
              style={{ color: 'var(--text-tertiary)' }}
              onMouseEnter={(e) => (e.currentTarget.style.color = 'var(--text-primary)')}
              onMouseLeave={(e) => (e.currentTarget.style.color = 'var(--text-tertiary)')}
            >
              Email
            </Link>
          </div>
        </motion.div>
      </div>
    </footer>
  );
}

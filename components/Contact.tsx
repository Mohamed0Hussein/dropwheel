'use client';

import { motion } from 'framer-motion';
import Link from 'next/link';
import { ArrowUpRight } from 'lucide-react';

export default function Contact() {
  return (
    <section id="contact" className="py-32 px-6 md:px-16 lg:px-24 min-h-[60vh] flex items-center">
      <div className="max-w-6xl mx-auto w-full">
        <motion.div
          initial={{ opacity: 0, y: 30 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.6 }}
          className="text-center"
        >
          <span className="text-sm uppercase tracking-widest mb-6 block" style={{ color: 'var(--text-secondary)' }}>Contact</span>
          
          <h2 className="font-display text-4xl sm:text-5xl md:text-6xl lg:text-7xl mb-8 leading-tight" style={{ color: 'var(--text-primary)' }}>
            Have a project in mind?
          </h2>
          
          <p className="text-lg max-w-xl mx-auto mb-12" style={{ color: 'var(--text-tertiary)' }}>
            I&apos;m always open to discussing new projects, creative ideas, or opportunities to be part of your vision.
          </p>

          <div className="flex flex-col sm:flex-row items-center justify-center gap-6">
            <Link
              href="mailto:mohamed.hussein0x1@gmail.com"
              className="group inline-flex items-center gap-3 text-2xl md:text-3xl hover:text-gradient transition-all"
              style={{ color: 'var(--text-primary)' }}
            >
              <span className="pb-1" style={{ borderBottom: '2px solid var(--text-primary)' }}>
                mohamed.hussein0x1@gmail.com
              </span>
              <ArrowUpRight className="w-6 h-6 group-hover:rotate-45 transition-transform" />
            </Link>
          </div>

          {/* Social Links */}
          <motion.div
            initial={{ opacity: 0 }}
            whileInView={{ opacity: 1 }}
            viewport={{ once: true }}
            transition={{ delay: 0.3 }}
            className="mt-16 flex items-center justify-center gap-8"
          >
            <Link
              href="https://github.com/Mohamed0Hussein"
              target="_blank"
              rel="noopener noreferrer"
              className="text-sm uppercase tracking-wider transition-colors"
              style={{ color: 'var(--text-tertiary)' }}
              onMouseEnter={(e) => e.currentTarget.style.color = 'var(--text-primary)'}
              onMouseLeave={(e) => e.currentTarget.style.color = 'var(--text-tertiary)'}
            >
              GitHub
            </Link>
            <span style={{ color: 'var(--border-primary)' }}>•</span>
            <Link
              href="https://www.linkedin.com/in/mohamed-hussein-0x1/"
              target="_blank"
              rel="noopener noreferrer"
              className="text-sm uppercase tracking-wider transition-colors"
              style={{ color: 'var(--text-tertiary)' }}
              onMouseEnter={(e) => e.currentTarget.style.color = 'var(--text-primary)'}
              onMouseLeave={(e) => e.currentTarget.style.color = 'var(--text-tertiary)'}
            >
              LinkedIn
            </Link>
            <span style={{ color: 'var(--border-primary)' }}>•</span>
            <Link
              href="tel:+201149247110"
              className="text-sm uppercase tracking-wider transition-colors"
              style={{ color: 'var(--text-tertiary)' }}
              onMouseEnter={(e) => e.currentTarget.style.color = 'var(--text-primary)'}
              onMouseLeave={(e) => e.currentTarget.style.color = 'var(--text-tertiary)'}
            >
              Phone
            </Link>
          </motion.div>

          {/* Footer */}
          <motion.div
            initial={{ opacity: 0 }}
            whileInView={{ opacity: 1 }}
            viewport={{ once: true }}
            transition={{ delay: 0.5 }}
            className="mt-24 pt-8"
            style={{ borderTop: '1px solid var(--border-secondary)' }}
          >
            <p className="text-sm" style={{ color: 'var(--text-tertiary)' }}>
              © {new Date().getFullYear()} Mohamed Hussein. All rights reserved.
            </p>
          </motion.div>
        </motion.div>
      </div>
    </section>
  );
}

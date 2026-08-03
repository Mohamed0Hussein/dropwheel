'use client';

import { motion } from 'framer-motion';
import Link from 'next/link';
import { ArrowUpRight, Mail, Linkedin, Github, Phone, Sparkles, MessageCircle } from 'lucide-react';
import ContactForm from './ContactForm';

const socials = [
  { name: 'Email', href: 'mailto:mohamed.hussein0x1@gmail.com', icon: Mail, color: '#7c3aed' },
  { name: 'LinkedIn', href: 'https://www.linkedin.com/in/mohamed-hussein-0x1/', icon: Linkedin, external: true, color: '#3b82f6' },
  { name: 'GitHub', href: 'https://github.com/Mohamed0Hussein', icon: Github, external: true, color: '#0891b2' },
  { name: 'Phone', href: 'tel:+201149247110', icon: Phone, color: '#14b8a6' },
];

export default function Contact() {
  return (
    <section id="contact" className="py-28 md:py-40 px-6 md:px-8 relative z-10 overflow-hidden">

      {/* Large ambient background */}
      <div
        className="absolute -top-40 left-1/2 -translate-x-1/2 w-[1000px] h-[600px] rounded-full pointer-events-none -z-10"
        style={{
          background: 'radial-gradient(ellipse, var(--orb-1) 0%, var(--orb-2) 40%, transparent 70%)',
          filter: 'blur(100px)',
          opacity: 0.4,
        }}
      />
      <div
        className="absolute -bottom-20 -right-20 w-[500px] h-[500px] rounded-full pointer-events-none -z-10"
        style={{
          background: 'radial-gradient(circle, var(--orb-3) 0%, transparent 70%)',
          filter: 'blur(80px)',
          opacity: 0.3,
        }}
      />

      <div className="max-w-6xl mx-auto">
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-16 lg:gap-24 items-start">

          {/* ── Left: CTA ── */}
          <motion.div
            initial={{ opacity: 0, y: 32 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true, margin: '-80px' }}
            transition={{ duration: 0.9, ease: [0.25, 1, 0.5, 1] }}
          >
            <span className="section-label">Contact</span>
            <h2 className="section-title text-5xl md:text-6xl lg:text-7xl mb-6 tracking-tighter">
              Let&apos;s build <br />
              <span className="text-gradient">something great.</span>
            </h2>
            <p className="section-subtitle mb-10 text-lg md:text-xl font-light">
              I&apos;m currently open to new opportunities. Whether you have a question, a project
              idea, or just want to say hi — my inbox is always open.
            </p>

            {/* Primary CTA */}
            <div className="flex flex-wrap gap-4 mb-10">
              <motion.div
                whileHover={{ scale: 1.04 }}
                whileTap={{ scale: 0.96 }}
                className="inline-block"
              >
                <Link
                  href="mailto:mohamed.hussein0x1@gmail.com"
                  className="btn-primary shadow-xl"
                  aria-label="Send me an email"
                >
                  <Mail className="w-5 h-5" />
                  Say Hello
                  <ArrowUpRight className="w-4 h-4" />
                </Link>
              </motion.div>
              <motion.div
                whileHover={{ scale: 1.04 }}
                whileTap={{ scale: 0.96 }}
                className="inline-block"
              >
                <Link
                  href="https://www.linkedin.com/in/mohamed-hussein-0x1/"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="btn-outline"
                  aria-label="Connect on LinkedIn"
                >
                  <MessageCircle className="w-4 h-4" />
                  Let&apos;s Connect
                </Link>
              </motion.div>
            </div>

            {/* Availability note */}
            <div className="flex items-center gap-3 flex-wrap">
              <motion.div
                className="flex items-center gap-2 px-4 py-2 rounded-full text-sm font-semibold"
                style={{
                  background: 'rgba(34,197,94,0.08)',
                  border: '1px solid rgba(34,197,94,0.2)',
                  color: '#22c55e',
                }}
                animate={{ boxShadow: ['0 0 0 0 rgba(34,197,94,0)', '0 0 0 6px rgba(34,197,94,0.1)', '0 0 0 0 rgba(34,197,94,0)'] }}
                transition={{ duration: 3, repeat: Infinity }}
              >
                <span className="w-2 h-2 rounded-full bg-green-400 animate-pulse" />
                Open to opportunities
              </motion.div>
              <span className="text-sm" style={{ color: 'var(--text-tertiary)' }}>
                · Typically replies within 24h
              </span>
            </div>

            {/* Compact social icon row */}
            <div className="flex items-center gap-3 mt-8">
              {socials.map((social) => (
                <motion.a
                  key={social.name}
                  href={social.href}
                  target={social.external ? '_blank' : undefined}
                  rel={social.external ? 'noopener noreferrer' : undefined}
                  aria-label={social.name}
                  className="flex items-center justify-center w-11 h-11 rounded-xl"
                  style={{
                    background: `${social.color}12`,
                    border: `1px solid ${social.color}30`,
                    color: social.color,
                  }}
                  whileHover={{ scale: 1.1, y: -2 }}
                  whileTap={{ scale: 0.92 }}
                >
                  <social.icon className="w-4 h-4" />
                </motion.a>
              ))}
            </div>

            {/* Decorative sparkle accent */}
            <motion.div
              animate={{ rotate: [0, 20, 0, -20, 0], scale: [1, 1.15, 1] }}
              transition={{ duration: 7, repeat: Infinity, ease: 'easeInOut' }}
              className="absolute -left-6 top-1/2 hidden xl:block"
              aria-hidden
            >
              <Sparkles className="w-8 h-8 opacity-20" style={{ color: 'var(--accent)' }} />
            </motion.div>
          </motion.div>

          {/* ── Right: Contact form ── */}
          <motion.div
            initial={{ opacity: 0, y: 32 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true, margin: '-80px' }}
            transition={{ duration: 0.9, delay: 0.15, ease: [0.25, 1, 0.5, 1] }}
          >
            <ContactForm />
          </motion.div>
        </div>
      </div>
    </section>
  );
}

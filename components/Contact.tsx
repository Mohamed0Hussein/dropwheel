'use client';

import { motion } from 'framer-motion';
import Link from 'next/link';
import { ArrowUpRight, Mail, Linkedin, Github, Phone, Sparkles } from 'lucide-react';

const socials = [
  {
    name: 'Email',
    href: 'mailto:mohamed.hussein0x1@gmail.com',
    icon: Mail,
    label: 'mohamed.hussein0x1@gmail.com',
    external: false,
    color: '#6366f1',
  },
  {
    name: 'LinkedIn',
    href: 'https://www.linkedin.com/in/mohamed-hussein-0x1/',
    icon: Linkedin,
    label: 'linkedin.com/in/mohamed-hussein-0x1',
    external: true,
    color: '#0077b5',
  },
  {
    name: 'GitHub',
    href: 'https://github.com/Mohamed0Hussein',
    icon: Github,
    label: 'github.com/Mohamed0Hussein',
    external: true,
    color: '#8b5cf6',
  },
  {
    name: 'Phone',
    href: 'tel:+201149247110',
    icon: Phone,
    label: '+20 114 924 7110',
    external: false,
    color: '#10b981',
  },
];

const cardVariants = {
  hidden:  { opacity: 0, y: 20 },
  visible: (i: number) => ({
    opacity: 1,
    y: 0,
    transition: { duration: 0.5, delay: 0.3 + i * 0.1, ease: [0.25, 1, 0.5, 1] },
  }),
};

export default function Contact() {
  return (
    <section id="contact" className="py-24 md:py-36 px-6 md:px-8 relative z-10 overflow-hidden">

      {/* Large ambient background */}
      <div
        className="absolute -top-32 left-1/2 -translate-x-1/2 w-[800px] h-[500px] rounded-full blur-3xl pointer-events-none -z-10 opacity-30"
        style={{ background: 'radial-gradient(ellipse, var(--mesh-1), var(--mesh-2), transparent)' }}
      />

      <div className="max-w-6xl mx-auto">
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-16 lg:gap-24 items-start">

          {/* ── Left: CTA ── */}
          <motion.div
            initial={{ opacity: 0, y: 28 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true, margin: '-80px' }}
            transition={{ duration: 0.8, ease: [0.25, 1, 0.5, 1] }}
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

            {/* Availability note */}
            <div className="mt-8 flex items-center gap-3">
              <div
                className="flex items-center gap-2 px-3 py-1.5 rounded-full text-sm font-semibold"
                style={{
                  background: 'rgba(34,197,94,0.08)',
                  border: '1px solid rgba(34,197,94,0.2)',
                  color: '#22c55e',
                }}
              >
                <span className="w-2 h-2 rounded-full bg-green-400 animate-pulse" />
                Open to opportunities
              </div>
              <span className="text-sm" style={{ color: 'var(--text-tertiary)' }}>
                · Typically replies within 24 h
              </span>
            </div>

            {/* Decorative sparkle accent */}
            <motion.div
              animate={{ rotate: [0, 15, 0, -15, 0], scale: [1, 1.1, 1] }}
              transition={{ duration: 6, repeat: Infinity, ease: 'easeInOut' }}
              className="absolute -left-6 top-1/2 hidden xl:block"
              aria-hidden
            >
              <Sparkles className="w-8 h-8 opacity-20" style={{ color: 'var(--accent)' }} />
            </motion.div>
          </motion.div>

          {/* ── Right: Social links ── */}
          <div className="flex flex-col gap-3.5">
            {socials.map((social, index) => (
              <motion.div
                key={social.name}
                variants={cardVariants}
                initial="hidden"
                whileInView="visible"
                viewport={{ once: true }}
                custom={index}
              >
                <motion.div whileHover={{ x: 4 }} transition={{ type: 'spring', stiffness: 400, damping: 20 }}>
                  <Link
                    href={social.href}
                    target={social.external ? '_blank' : undefined}
                    rel={social.external ? 'noopener noreferrer' : undefined}
                    className="card group flex items-center gap-5 p-5 hover:shadow-xl transition-shadow duration-400"
                    id={`contact-${social.name.toLowerCase()}`}
                    aria-label={`Connect on ${social.name}`}
                  >
                    {/* Icon */}
                    <div
                      className="w-11 h-11 rounded-xl flex items-center justify-center shrink-0 transition-all duration-400 group-hover:scale-110"
                      style={{
                        background: `${social.color}15`,
                        border: `1px solid ${social.color}30`,
                      }}
                    >
                      <social.icon
                        className="w-5 h-5 transition-colors duration-300"
                        style={{ color: social.color }}
                      />
                    </div>

                    {/* Text */}
                    <div className="flex-1 min-w-0">
                      <p
                        className="text-[0.65rem] font-bold uppercase tracking-[0.2em] mb-0.5"
                        style={{ color: 'var(--text-tertiary)' }}
                      >
                        {social.name}
                      </p>
                      <p
                        className="text-sm font-semibold truncate transition-colors duration-300 group-hover:text-accent"
                        style={{ color: 'var(--text-primary)' }}
                      >
                        {social.label}
                      </p>
                    </div>

                    {/* Arrow */}
                    <ArrowUpRight
                      className="w-4 h-4 shrink-0 opacity-0 -translate-x-2 group-hover:opacity-100 group-hover:translate-x-0 transition-all duration-300"
                      style={{ color: 'var(--accent)' }}
                    />
                  </Link>
                </motion.div>
              </motion.div>
            ))}
          </div>
        </div>
      </div>
    </section>
  );
}

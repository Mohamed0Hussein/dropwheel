'use client';

import { motion } from 'framer-motion';
import Link from 'next/link';
import { ArrowUpRight, Mail, Linkedin, Github, Phone } from 'lucide-react';

const socials = [
  {
    name: 'Email',
    href: 'mailto:mohamed.hussein0x1@gmail.com',
    icon: Mail,
    label: 'mohamed.hussein0x1@gmail.com',
    external: false,
  },
  {
    name: 'LinkedIn',
    href: 'https://www.linkedin.com/in/mohamed-hussein-0x1/',
    icon: Linkedin,
    label: 'linkedin.com/in/mohamed-hussein-0x1',
    external: true,
  },
  {
    name: 'GitHub',
    href: 'https://github.com/Mohamed0Hussein',
    icon: Github,
    label: 'github.com/Mohamed0Hussein',
    external: true,
  },
  {
    name: 'Phone',
    href: 'tel:+201149247110',
    icon: Phone,
    label: '+20 114 924 7110',
    external: false,
  },
];

export default function Contact() {
  return (
    <section id="contact" className="py-24 md:py-32 px-6 md:px-8">
      <div className="max-w-6xl mx-auto">
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-12 lg:gap-16">
          {/* Left: CTA */}
          <motion.div
            initial={{ opacity: 0, y: 24 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.6 }}
          >
            <span className="section-label">Contact</span>
            <h2 className="section-title text-4xl md:text-5xl lg:text-6xl mb-6">
              Let&apos;s build
              <br />
              <span className="text-gradient">something great.</span>
            </h2>
            <p className="section-subtitle mb-8">
              I&apos;m always open to discussing new projects, creative ideas, or
              opportunities to bring your vision to life. Let&apos;s connect.
            </p>
            <Link
              href="mailto:mohamed.hussein0x1@gmail.com"
              className="btn-primary"
            >
              <Mail className="w-4 h-4" />
              Send me an email
              <ArrowUpRight className="w-4 h-4" />
            </Link>
          </motion.div>

          {/* Right: Social Links */}
          <motion.div
            initial={{ opacity: 0, y: 24 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.6, delay: 0.15 }}
            className="flex flex-col justify-center"
          >
            <div className="space-y-3">
              {socials.map((social, index) => (
                <motion.div
                  key={social.name}
                  initial={{ opacity: 0, x: 16 }}
                  whileInView={{ opacity: 1, x: 0 }}
                  viewport={{ once: true }}
                  transition={{ duration: 0.4, delay: 0.2 + index * 0.08 }}
                >
                  <Link
                    href={social.href}
                    target={social.external ? '_blank' : undefined}
                    rel={social.external ? 'noopener noreferrer' : undefined}
                    className="card flex items-center gap-4 p-4 md:p-5 group"
                    style={{ borderRadius: '14px' }}
                    id={`contact-${social.name.toLowerCase()}`}
                  >
                    <div
                      className="w-10 h-10 rounded-xl flex items-center justify-center shrink-0"
                      style={{
                        background: 'var(--badge-bg)',
                        border: '1px solid var(--badge-border)',
                      }}
                    >
                      <social.icon
                        className="w-[18px] h-[18px]"
                        style={{ color: 'var(--accent)' }}
                      />
                    </div>
                    <div className="flex-1 min-w-0">
                      <p
                        className="text-xs font-semibold uppercase tracking-wider mb-0.5"
                        style={{ color: 'var(--text-tertiary)' }}
                      >
                        {social.name}
                      </p>
                      <p
                        className="text-sm font-medium truncate"
                        style={{ color: 'var(--text-primary)' }}
                      >
                        {social.label}
                      </p>
                    </div>
                    <ArrowUpRight
                      className="w-4 h-4 shrink-0 opacity-0 group-hover:opacity-100 transition-all duration-300"
                      style={{ color: 'var(--accent)' }}
                    />
                  </Link>
                </motion.div>
              ))}
            </div>
          </motion.div>
        </div>
      </div>
    </section>
  );
}

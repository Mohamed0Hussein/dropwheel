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
    <section id="contact" className="py-24 md:py-36 px-6 md:px-8 relative z-10">
      <div className="max-w-6xl mx-auto">
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-16 lg:gap-24 items-center">
          {/* Left: CTA */}
          <motion.div
            initial={{ opacity: 0, x: -30 }}
            whileInView={{ opacity: 1, x: 0 }}
            viewport={{ once: true, margin: '-100px' }}
            transition={{ duration: 0.8, ease: [0.25, 1, 0.5, 1] }}
            className="relative"
          >
            <div className="absolute -top-20 -left-20 w-64 h-64 bg-accent opacity-10 blur-3xl rounded-full pointer-events-none -z-10" />
            
            <span className="section-label">
              <span className="inline-block w-2h-2 rounded-full bg-accent mr-3 animate-pulse"></span>
              Contact
            </span>
            <h2 className="section-title text-5xl md:text-6xl lg:text-7xl mb-6 tracking-tighter">
              Let&apos;s build <br />
              <span className="text-gradient">something great.</span>
            </h2>
            <p className="section-subtitle mb-10 text-lg md:text-xl font-light">
              I&apos;m currently open to new opportunities. Whether you have a question, a project idea, or just want to say hi, my inbox is always open.
            </p>
            <Link
              href="mailto:mohamed.hussein0x1@gmail.com"
              className="btn-primary inline-flex scale-100 hover:scale-[1.03] transition-transform duration-500 shadow-xl"
            >
              <Mail className="w-5 h-5 mr-1" />
              Say Hello
              <ArrowUpRight className="w-5 h-5 ml-1" />
            </Link>
          </motion.div>

          {/* Right: Social Links */}
          <motion.div
            initial={{ opacity: 0, x: 30 }}
            whileInView={{ opacity: 1, x: 0 }}
            viewport={{ once: true, margin: '-100px' }}
            transition={{ duration: 0.8, delay: 0.2, ease: [0.25, 1, 0.5, 1] }}
            className="flex flex-col justify-center gap-4 relative"
          >
            <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-full h-full bg-linear-to-br from-accent/5 via-transparent to-transparent opacity-50 blur-2xl rounded-full pointer-events-none -z-10" />

            {socials.map((social, index) => (
              <motion.div
                key={social.name}
                initial={{ opacity: 0, y: 20 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                transition={{ duration: 0.5, delay: 0.3 + index * 0.1, ease: [0.25, 1, 0.5, 1] }}
              >
                <Link
                  href={social.href}
                  target={social.external ? '_blank' : undefined}
                  rel={social.external ? 'noopener noreferrer' : undefined}
                  className="card flex items-center gap-5 p-5 md:p-6 group hover:-translate-y-1 hover:shadow-xl transition-all duration-400"
                  style={{ borderRadius: '20px' }}
                  id={`contact-${social.name.toLowerCase()}`}
                  aria-label={social.name}
                >
                  <div
                    className="w-12 h-12 rounded-xl flex items-center justify-center shrink-0 transition-transform duration-500 group-hover:scale-110 shadow-sm"
                    style={{
                      background: 'var(--badge-bg)',
                      border: '1px solid var(--badge-border)',
                    }}
                  >
                    <social.icon
                      className="w-5 h-5 group-hover:text-accent transition-colors duration-300"
                      style={{ color: 'var(--text-secondary)' }}
                    />
                  </div>
                  <div className="flex-1 min-w-0">
                    <p
                      className="text-[0.7rem] font-bold uppercase tracking-[0.2em] mb-1 group-hover:text-accent transition-colors duration-300"
                      style={{ color: 'var(--text-tertiary)' }}
                    >
                      {social.name}
                    </p>
                    <p
                      className="text-base font-semibold truncate transition-colors duration-300"
                      style={{ color: 'var(--text-primary)' }}
                    >
                      {social.label}
                    </p>
                  </div>
                  <ArrowUpRight
                    className="w-5 h-5 shrink-0 transition-all duration-300 text-text-tertiary opacity-0 -translate-x-4 group-hover:opacity-100 group-hover:translate-x-0 group-hover:text-accent"
                  />
                </Link>
              </motion.div>
            ))}
          </motion.div>
        </div>
      </div>
    </section>
  );
}

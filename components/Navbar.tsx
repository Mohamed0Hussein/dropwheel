'use client';

import { useState, useEffect } from 'react';
import { motion, AnimatePresence, useScroll, useMotionValueEvent } from 'framer-motion';
import { Menu, X } from 'lucide-react';
import Link from 'next/link';
import ThemeToggle from './ThemeToggle';

const navLinks = [
  { name: 'About', href: '#about' },
  { name: 'Projects', href: '#projects' },
  { name: 'Stack', href: '#stack' },
  { name: 'Experience', href: '#experience' },
  { name: 'Contact', href: '#contact' },
];

export default function Navbar() {
  const [isOpen, setIsOpen] = useState(false);
  const [isScrolled, setIsScrolled] = useState(false);
  const [hoveredLink, setHoveredLink] = useState<string | null>(null);
  const { scrollY } = useScroll();

  useMotionValueEvent(scrollY, 'change', (latest) => {
    setIsScrolled(latest > 50);
  });

  useEffect(() => {
    if (isOpen) {
      document.body.style.overflow = 'hidden';
    } else {
      document.body.style.overflow = '';
    }
    return () => { document.body.style.overflow = ''; };
  }, [isOpen]);

  return (
    <>
      <motion.header
        initial={{ y: -20, opacity: 0 }}
        animate={{ y: 0, opacity: 1 }}
        transition={{ duration: 0.6, delay: 0.2, ease: [0.25, 1, 0.5, 1] }}
        className={`fixed top-0 left-0 right-0 z-50 transition-all duration-500 pt-4 md:pt-6 px-4 md:px-8`}
      >
        <div className="max-w-6xl mx-auto">
          <div
            className={`flex items-center justify-between transition-all duration-500 ${
              isScrolled
                ? 'glass px-5 py-3 rounded-2xl shadow-lg border border-white/10'
                : 'bg-transparent px-2 py-3 border-transparent'
            }`}
          >
            {/* Logo */}
            <Link href="/" className="relative group">
              <span
                className="text-lg font-display font-bold tracking-tight"
                style={{ color: 'var(--text-primary)' }}
              >
                Mohamed <span className="text-gradient">Hussein</span>
                <span className="absolute -bottom-1 left-0 w-0 h-0.5 bg-accent transition-all duration-300 group-hover:w-full"></span>
              </span>
            </Link>

            {/* Desktop Nav Links */}
            <nav 
              className="hidden md:flex items-center gap-1.5 p-1 rounded-full border border-transparent transition-all"
              style={{ background: isScrolled ? 'rgba(0,0,0,0.02)' : 'transparent' }}
              onMouseLeave={() => setHoveredLink(null)}
              aria-label="Main navigation"
            >
              {navLinks.map((link) => (
                <Link
                  key={link.name}
                  href={link.href}
                  className="relative px-5 py-2.5 rounded-full text-sm font-semibold transition-colors duration-300 outline-none"
                  style={{ color: hoveredLink === link.name ? 'var(--text-primary)' : 'var(--text-secondary)' }}
                  onMouseEnter={() => setHoveredLink(link.name)}
                  onClick={() => setHoveredLink(link.name)}
                >
                  {hoveredLink === link.name && (
                    <motion.div
                      layoutId="nav-hover"
                      className="absolute inset-0 rounded-full z-0"
                      style={{ background: 'var(--surface-hover)' }}
                      initial={false}
                      transition={{ type: 'spring', stiffness: 450, damping: 30 }}
                    />
                  )}
                  <span className="relative z-10">{link.name}</span>
                </Link>
              ))}
            </nav>

            {/* Right Side */}
            <div className="flex items-center gap-3">
              <ThemeToggle />
              <Link
                href="#contact"
                className="hidden md:inline-flex btn-primary py-2.5! px-6! text-sm! font-bold!"
              >
                Let&apos;s Talk
              </Link>
              <button
                onClick={() => setIsOpen(!isOpen)}
                className="md:hidden glass p-2.5 rounded-xl transition-all hover:scale-105 active:scale-95"
                aria-label="Toggle menu"
                id="mobile-menu-toggle"
              >
                {isOpen ? <X size={20} /> : <Menu size={20} />}
              </button>
            </div>
          </div>
        </div>
      </motion.header>

      {/* Mobile Menu */}
      <AnimatePresence>
        {isOpen && (
          <motion.div
            initial={{ opacity: 0, backdropFilter: 'blur(0px)' }}
            animate={{ opacity: 1, backdropFilter: 'blur(24px)' }}
            exit={{ opacity: 0, backdropFilter: 'blur(0px)' }}
            transition={{ duration: 0.4 }}
            className="fixed inset-0 z-40 flex items-center justify-center"
            style={{ background: 'var(--glass-bg)' }}
          >
            <nav className="flex flex-col items-center gap-8 w-full px-6" aria-label="Mobile navigation">
              {navLinks.map((link, index) => (
                <motion.div
                  key={link.name}
                  initial={{ opacity: 0, y: 30 }}
                  animate={{ opacity: 1, y: 0 }}
                  exit={{ opacity: 0, y: -20 }}
                  transition={{ delay: index * 0.1, duration: 0.4, ease: [0.25, 1, 0.5, 1] }}
                >
                  <Link
                    href={link.href}
                    onClick={() => setIsOpen(false)}
                    className="text-4xl font-display font-bold transition-all duration-300 relative group overflow-hidden"
                    style={{ color: 'var(--text-primary)', letterSpacing: '-0.02em' }}
                  >
                    {link.name}
                    <span className="absolute -bottom-2 left-0 w-0 h-1 bg-accent transition-all duration-300 group-hover:w-full"></span>
                  </Link>
                </motion.div>
              ))}
              <motion.div
                initial={{ opacity: 0, scale: 0.9 }}
                animate={{ opacity: 1, scale: 1 }}
                exit={{ opacity: 0, scale: 0.9 }}
                transition={{ delay: navLinks.length * 0.1, duration: 0.4 }}
                className="mt-8 w-full max-w-xs"
              >
                <Link
                  href="#contact"
                  onClick={() => setIsOpen(false)}
                  className="btn-primary w-full shadow-lg"
                >
                  Let&apos;s Talk
                </Link>
              </motion.div>
            </nav>
          </motion.div>
        )}
      </AnimatePresence>
    </>
  );
}

'use client';

import { useState, useEffect, useRef } from 'react';
import { motion, AnimatePresence, useScroll, useMotionValueEvent, Variants } from 'framer-motion';
import { Menu, X } from 'lucide-react';
import Link from 'next/link';
import ThemeToggle from './ThemeToggle';
import { useActiveSection } from '@/hooks/useActiveSection';

const navLinks = [
  { name: 'About',      href: '#about'      },
  { name: 'Projects',   href: '#projects'   },
  { name: 'Stack',      href: '#stack'      },
  { name: 'Experience', href: '#experience' },
  { name: 'Contact',    href: '#contact'    },
];

const mobileMenuVariants: Variants = {
  hidden: { opacity: 0, clipPath: 'inset(0 0 100% 0 round 16px)' },
  visible: {
    opacity: 1,
    clipPath: 'inset(0 0 0% 0 round 0px)',
    transition: { duration: 0.45, ease: [0.25, 1, 0.5, 1] },
  },
  exit: {
    opacity: 0,
    clipPath: 'inset(0 0 100% 0 round 16px)',
    transition: { duration: 0.35, ease: [0.5, 0, 0.75, 0] },
  },
};

const linkStagger = {
  hidden:  {},
  visible: { transition: { staggerChildren: 0.07, delayChildren: 0.1 } },
  exit:    { transition: { staggerChildren: 0.04, staggerDirection: -1 } },
};

const linkItem: Variants = {
  hidden:  { opacity: 0, y: 24 },
  visible: { opacity: 1, y: 0,  transition: { duration: 0.4, ease: [0.25, 1, 0.5, 1] } },
  exit:    { opacity: 0, y: -16, transition: { duration: 0.25 } },
};

export default function Navbar() {
  const [isOpen, setIsOpen]       = useState(false);
  const [isScrolled, setIsScrolled] = useState(false);
  const [hoveredLink, setHoveredLink] = useState<string | null>(null);
  const activeSection = useActiveSection();
  const { scrollY } = useScroll();
  const menuToggleRef = useRef<HTMLButtonElement>(null);
  const firstMobileLinkRef = useRef<HTMLAnchorElement>(null);

  useMotionValueEvent(scrollY, 'change', (latest) => {
    setIsScrolled(latest > 50);
  });

  useEffect(() => {
    document.body.style.overflow = isOpen ? 'hidden' : '';
    if (isOpen) {
      // Move focus into the overlay once its enter animation mounts it.
      const id = requestAnimationFrame(() => firstMobileLinkRef.current?.focus());
      return () => cancelAnimationFrame(id);
    }
    return () => { document.body.style.overflow = ''; };
  }, [isOpen]);

  useEffect(() => {
    if (!isOpen) return;
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        setIsOpen(false);
        menuToggleRef.current?.focus();
      }
    };
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, [isOpen]);

  return (
    <>
      <motion.header
        initial={{ y: -24, opacity: 0 }}
        animate={{ y: 0,   opacity: 1 }}
        transition={{ duration: 0.7, delay: 0.2, ease: [0.25, 1, 0.5, 1] }}
        className="fixed top-0 left-0 right-0 z-50 pt-4 md:pt-5 px-4 md:px-8"
      >
        <div className="max-w-6xl mx-auto">
          <motion.div
            animate={isScrolled
              ? ({ backdropFilter: 'blur(24px)', WebkitBackdropFilter: 'blur(24px)' } as any)
              : ({ backdropFilter: 'blur(0px)', WebkitBackdropFilter: 'blur(0px)' } as any)
            }
            transition={{ duration: 0.4 }}
            className={`flex items-center justify-between transition-all duration-500 ${
              isScrolled
                ? 'glass px-5 py-3 rounded-2xl shadow-lg'
                : 'bg-transparent px-2 py-3'
            }`}
          >
            {/* Logo */}
            <Link href="/" className="relative group outline-none" aria-label="Home">
              <span
                className="text-lg font-display font-bold tracking-tight"
                style={{ color: 'var(--text-primary)' }}
              >
                Mohamed{' '}
                <span className="text-gradient">Hussein</span>
              </span>
              <span className="absolute -bottom-0.5 left-0 h-px w-0 bg-accent transition-all duration-300 group-hover:w-full rounded-full" />
            </Link>

            {/* Desktop nav */}
            <nav
              className="hidden md:flex items-center gap-1 p-1 rounded-full"
              onMouseLeave={() => setHoveredLink(null)}
              aria-label="Main navigation"
            >
              {navLinks.map((link) => {
                const sectionId = link.href.slice(1);
                const isActive  = activeSection === sectionId;
                return (
                  <Link
                    key={link.name}
                    href={link.href}
                    className="relative px-4 py-2.5 rounded-full text-sm font-semibold transition-colors duration-300 outline-none"
                    style={{
                      color: hoveredLink === link.name || isActive
                        ? 'var(--text-primary)'
                        : 'var(--text-secondary)',
                    }}
                    onMouseEnter={() => setHoveredLink(link.name)}
                    onClick={() => setHoveredLink(null)}
                  >
                    {/* Hover/active pill */}
                    {hoveredLink === link.name && (
                      <motion.div
                        layoutId="nav-hover"
                        className="absolute inset-0 rounded-full z-0"
                        style={{ background: 'var(--surface-hover)' }}
                        initial={false}
                        transition={{ type: 'spring', stiffness: 500, damping: 32 }}
                      />
                    )}

                    <span className="relative z-10">{link.name}</span>

                    {/* Active dot */}
                    {isActive && hoveredLink !== link.name && (
                      <motion.span
                        layoutId="nav-active"
                        className="absolute -bottom-0.5 left-1/2 -translate-x-1/2 nav-active-dot"
                        initial={false}
                        transition={{ type: 'spring', stiffness: 500, damping: 30 }}
                      />
                    )}
                  </Link>
                );
              })}
            </nav>

            {/* Right side */}
            <div className="flex items-center gap-3">
              <ThemeToggle />
              <motion.div whileHover={{ scale: 1.04 }} whileTap={{ scale: 0.96 }}>
                <Link
                  href="#contact"
                  className="hidden md:inline-flex btn-primary py-2.5! px-6! text-sm! font-bold!"
                  aria-label="Let's Talk"
                >
                  Let&apos;s Talk
                </Link>
              </motion.div>
              <button
                ref={menuToggleRef}
                onClick={() => setIsOpen(!isOpen)}
                className="md:hidden glass p-2.5 rounded-xl transition-all hover:scale-105 active:scale-95"
                aria-label={isOpen ? 'Close menu' : 'Open menu'}
                aria-expanded={isOpen}
                aria-controls="mobile-nav-menu"
                id="mobile-menu-toggle"
              >
                <AnimatePresence mode="wait" initial={false}>
                  {isOpen
                    ? <motion.div key="x" initial={{ rotate: -90, opacity: 0 }} animate={{ rotate: 0, opacity: 1 }} exit={{ rotate: 90, opacity: 0 }} transition={{ duration: 0.2 }}><X size={20} /></motion.div>
                    : <motion.div key="menu" initial={{ rotate: 90, opacity: 0 }} animate={{ rotate: 0, opacity: 1 }} exit={{ rotate: -90, opacity: 0 }} transition={{ duration: 0.2 }}><Menu size={20} /></motion.div>
                  }
                </AnimatePresence>
              </button>
            </div>
          </motion.div>
        </div>
      </motion.header>

      {/* ── Mobile Menu (full-screen overlay) ── */}
      <AnimatePresence>
        {isOpen && (
          <motion.div
            variants={mobileMenuVariants}
            initial="hidden"
            animate="visible"
            exit="exit"
            className="fixed inset-0 z-40 flex flex-col items-center justify-center"
            style={{ background: 'var(--glass-bg)', backdropFilter: 'blur(28px)', WebkitBackdropFilter: 'blur(28px)' }}
            id="mobile-nav-menu"
            role="dialog"
            aria-modal="true"
            aria-label="Mobile navigation"
          >
            {/* Accent blob */}
            <div className="absolute top-1/4 left-1/2 -translate-x-1/2 w-96 h-96 rounded-full blur-3xl pointer-events-none" style={{ background: 'var(--mesh-1)' }} />

            <motion.nav
              variants={linkStagger}
              initial="hidden"
              animate="visible"
              exit="exit"
              className="flex flex-col items-center gap-6 w-full px-8"
              aria-label="Mobile navigation"
            >
              {navLinks.map((link, i) => {
                const sectionId = link.href.slice(1);
                const isActive  = activeSection === sectionId;
                return (
                  <motion.div key={link.name} variants={linkItem}>
                    <Link
                      ref={i === 0 ? firstMobileLinkRef : undefined}
                      href={link.href}
                      onClick={() => setIsOpen(false)}
                      className="relative text-4xl font-display font-bold group"
                      style={{
                        color: isActive ? 'var(--accent)' : 'var(--text-primary)',
                        letterSpacing: '-0.02em',
                      }}
                    >
                      {link.name}
                      <span className="absolute -bottom-1 left-0 h-[2px] w-0 bg-accent rounded-full transition-all duration-300 group-hover:w-full" />
                    </Link>
                  </motion.div>
                );
              })}

              <motion.div
                variants={linkItem}
                className="mt-6 w-full max-w-xs"
              >
                <Link
                  href="#contact"
                  onClick={() => setIsOpen(false)}
                  className="btn-primary w-full justify-center shadow-xl"
                >
                  Let&apos;s Talk
                </Link>
              </motion.div>
            </motion.nav>
          </motion.div>
        )}
      </AnimatePresence>
    </>
  );
}

'use client';

import { useState, useEffect } from 'react';
import { motion } from 'framer-motion';
import { Menu, X, Github, Linkedin } from 'lucide-react';
import Link from 'next/link';
import clsx from 'clsx';

const navLinks = [
  { name: 'About', href: '#about' },
  { name: 'Skills', href: '#skills' },
  { name: 'Projects', href: '#projects' },
  { name: 'Contact', href: '#contact' },
];

export default function Navbar() {
  const [isOpen, setIsOpen] = useState(false);
  const [scrolled, setScrolled] = useState(false);

  useEffect(() => {
    const handleScroll = () => {
      setScrolled(window.scrollY > 50);
    };
    window.addEventListener('scroll', handleScroll);
    return () => window.removeEventListener('scroll', handleScroll);
  }, []);

  return (
    <motion.nav
      initial={{ y: -100 }}
      animate={{ y: 0 }}
      className={clsx(
        'fixed top-0 w-full z-50 transition-all duration-300 border-b border-transparent',
        scrolled ? 'glass py-4 border-white/10' : 'bg-transparent py-6'
      )}
    >
      <div className="container mx-auto px-6 flex justify-between items-center">
        <Link href="/" className="text-2xl font-bold bg-clip-text text-transparent bg-linear-to-r from-indigo-500 via-purple-500 to-pink-500 hover:opacity-80 transition-opacity">
          M.Hussein
        </Link>

        <div className="hidden md:flex items-center space-x-8">
          {navLinks.map((link) => (
            <Link
              key={link.name}
              href={link.href}
              className="text-gray-400 hover:text-white transition-colors text-sm font-medium tracking-wide"
            >
              {link.name}
            </Link>
          ))}
          <div className="flex items-center space-x-4 ml-4 border-l border-gray-700 pl-4">
            <Link href="https://github.com/Mohamed0Hussein" target="_blank" className="text-gray-400 hover:text-white transition-colors">
              <Github size={20} />
            </Link>
            <Link href="https://www.linkedin.com/in/mohamed-hussein-0x1/" target="_blank" className="text-gray-400 hover:text-white transition-colors">
              <Linkedin size={20} />
            </Link>
          </div>
        </div>

        <button
          className="md:hidden text-gray-300"
          onClick={() => setIsOpen(!isOpen)}
        >
          {isOpen ? <X /> : <Menu />}
        </button>
      </div>

      {isOpen && (
        <motion.div
          initial={{ opacity: 0, height: 0 }}
          animate={{ opacity: 1, height: 'auto' }}
          className="md:hidden glass border-t border-white/10"
        >
          <div className="px-6 py-4 flex flex-col space-y-4">
            {navLinks.map((link) => (
              <Link
                key={link.name}
                href={link.href}
                className="text-gray-300 hover:text-white block py-2"
                onClick={() => setIsOpen(false)}
              >
                {link.name}
              </Link>
            ))}
          </div>
        </motion.div>
      )}
    </motion.nav>
  );
}

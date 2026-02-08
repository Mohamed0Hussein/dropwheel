'use client';

import { Moon, Sun } from 'lucide-react';
import { motion } from 'framer-motion';
import { useEffect, useState } from 'react';

export default function ThemeToggle() {
  const [mounted, setMounted] = useState(false);
  const [currentTheme, setCurrentTheme] = useState<'dark' | 'light'>('dark');

  useEffect(() => {
    setMounted(true);
    const savedTheme = (localStorage.getItem('theme') as 'dark' | 'light') || 'dark';
    setCurrentTheme(savedTheme);
    document.documentElement.setAttribute('data-theme', savedTheme);
  }, []);

  const handleToggle = () => {
    const newTheme = currentTheme === 'dark' ? 'light' : 'dark';
    setCurrentTheme(newTheme);
    localStorage.setItem('theme', newTheme);
    document.documentElement.setAttribute('data-theme', newTheme);
  };

  if (!mounted) {
    return (
      <div className="theme-toggle" style={{ width: '49px', height: '49px', padding: '12px' }}>
        <div style={{ width: '20px', height: '20px' }} />
      </div>
    );
  }

  return (
    <motion.button
      onClick={handleToggle}
      className="theme-toggle group"
      aria-label="Toggle theme"
      whileHover={{ scale: 1.05 }}
      whileTap={{ scale: 0.95 }}
    >
      <motion.div
        className="theme-toggle-inner"
        initial={false}
        animate={{
          rotate: currentTheme === 'dark' ? 0 : 180,
        }}
        transition={{ duration: 0.5, ease: 'easeInOut' }}
      >
        {currentTheme === 'dark' ? (
          <Moon className="w-5 h-5" />
        ) : (
          <Sun className="w-5 h-5" />
        )}
      </motion.div>
      <span className="theme-toggle-bg"></span>
    </motion.button>
  );
}

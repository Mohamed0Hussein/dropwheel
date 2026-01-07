'use client';

import { useEffect, useState } from 'react';
import { motion } from 'framer-motion';

export default function SideElements() {
  const [scrollProgress, setScrollProgress] = useState(0);

  useEffect(() => {
    const handleScroll = () => {
      const totalHeight = document.documentElement.scrollHeight - window.innerHeight;
      const progress = (window.scrollY / totalHeight) * 100;
      setScrollProgress(progress);
    };

    window.addEventListener('scroll', handleScroll);
    return () => window.removeEventListener('scroll', handleScroll);
  }, []);

  return (
    <>
      {/* Left Side - Email */}
      <motion.div
        initial={{ opacity: 0, x: -20 }}
        animate={{ opacity: 1, x: 0 }}
        transition={{ delay: 1, duration: 0.6 }}
        className="fixed left-8 top-1/2 -translate-y-1/2 z-50 hidden lg:flex flex-col items-center gap-6"
      >
        {/* Scroll Progress */}
        <div className="h-24 w-[2px] bg-white/10 relative overflow-hidden rounded-full">
          <div 
            className="absolute top-0 left-0 w-full bg-green-400 transition-all duration-150 rounded-full"
            style={{ height: `${scrollProgress}%` }}
          />
        </div>
        
        {/* Status Dot */}
        <div className="flex items-center gap-2">
          <span className="w-2 h-2 rounded-full bg-green-400 animate-pulse" />
        </div>

        {/* Vertical Email */}
        <a 
          href="mailto:mohamed.hussein0x1@gmail.com"
          className="vertical-text text-xs tracking-widest text-gray-500 hover:text-white transition-colors"
        >
          mohamed.hussein0x1@gmail.com
        </a>
      </motion.div>

      {/* Right Side - Scroll Indicator */}
      <motion.div
        initial={{ opacity: 0, x: 20 }}
        animate={{ opacity: 1, x: 0 }}
        transition={{ delay: 1, duration: 0.6 }}
        className="fixed right-8 bottom-8 z-50 hidden lg:block"
      >
        <div className="flex flex-col items-center gap-2 text-gray-500 text-xs">
          <span className="vertical-text tracking-widest">SCROLL</span>
          <motion.div
            animate={{ y: [0, 8, 0] }}
            transition={{ duration: 1.5, repeat: Infinity }}
            className="w-[1px] h-8 bg-gradient-to-b from-white/50 to-transparent"
          />
        </div>
      </motion.div>
    </>
  );
}

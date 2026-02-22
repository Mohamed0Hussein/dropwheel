'use client';

import { motion, useScroll, useSpring, useTransform } from 'framer-motion';

export default function SideElements() {
  const { scrollYProgress } = useScroll();
  const scaleY = useSpring(scrollYProgress, {
    stiffness: 100,
    damping: 30,
    restDelta: 0.001
  });

  const emailOpacity = useTransform(scrollYProgress, [0.95, 1], [1, 0]);

  return (
    <>
      {/* Left: Scroll Progress + Email */}
      <motion.div
        initial={{ opacity: 0, x: -20, filter: 'blur(10px)' }}
        animate={{ opacity: 1, x: 0, filter: 'blur(0px)' }}
        transition={{ delay: 1.5, duration: 0.8, ease: [0.25, 1, 0.5, 1] }}
        className="fixed left-6 md:left-8 top-1/2 -translate-y-1/2 z-40 hidden xl:flex flex-col items-center gap-6"
      >
        <div
          className="h-24 w-[2px] relative overflow-hidden rounded-full shadow-sm"
          style={{ background: 'var(--border)' }}
        >
          <motion.div
            className="absolute top-0 left-0 w-full rounded-full origin-top"
            style={{
              scaleY,
              background: 'linear-gradient(180deg, var(--gradient-from), var(--gradient-to))',
              height: '100%',
            }}
          />
        </div>
        <div className="flex items-center">
          <span className="status-dot" style={{ width: '8px', height: '8px' }} />
        </div>
        <motion.a
          style={{ opacity: emailOpacity, color: 'var(--text-tertiary)' }}
          href="mailto:mohamed.hussein0x1@gmail.com"
          className="vertical-text text-[0.65rem] tracking-[0.25em] font-bold transition-colors duration-300 hover:text-accent"
        >
          mohamed.hussein0x1@gmail.com
        </motion.a>
      </motion.div>
    </>
  );
}

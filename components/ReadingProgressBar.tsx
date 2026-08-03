'use client';

import { motion, useScroll, useSpring } from 'framer-motion';

export default function ReadingProgressBar() {
  const { scrollYProgress } = useScroll();
  const scaleX = useSpring(scrollYProgress, {
    stiffness: 120,
    damping: 30,
    restDelta: 0.001,
  });

  return (
    <div className="reading-progress-track" aria-hidden="true">
      <motion.div className="reading-progress-fill" style={{ scaleX }} />
    </div>
  );
}

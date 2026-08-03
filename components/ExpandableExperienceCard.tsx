'use client';

import { AnimatePresence, motion } from 'framer-motion';
import { CheckCircle2, ChevronDown } from 'lucide-react';
import { useState } from 'react';

const COLLAPSED_COUNT = 2;

export default function ExpandableExperienceCard({
  achievements,
  accentColor,
  baseDelay,
}: {
  achievements: string[];
  accentColor: string;
  baseDelay: number;
}) {
  const [expanded, setExpanded] = useState(false);
  const visible = achievements.slice(0, COLLAPSED_COUNT);
  const hidden = achievements.slice(COLLAPSED_COUNT);

  return (
    <>
      <ul className="space-y-3 pt-5" style={{ borderTop: '1px solid var(--border)' }}>
        {visible.map((item, i) => (
          <motion.li
            key={i}
            initial={{ opacity: 0, x: -12 }}
            whileInView={{ opacity: 1, x: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.45, delay: baseDelay + i * 0.09 }}
            className="flex items-start gap-3 text-[0.9rem] leading-relaxed group/item"
            style={{ color: 'var(--text-secondary)' }}
          >
            <CheckCircle2
              className="w-4 h-4 shrink-0 mt-0.5 transition-all duration-300 group-hover/item:scale-110"
              style={{ color: accentColor, opacity: 0.75 }}
            />
            <span className="transition-colors duration-300 group-hover/item:text-[var(--text-primary)]">
              {item}
            </span>
          </motion.li>
        ))}

        <AnimatePresence initial={false}>
          {expanded &&
            hidden.map((item, i) => (
              <motion.li
                key={i}
                initial={{ opacity: 0, height: 0 }}
                animate={{ opacity: 1, height: 'auto' }}
                exit={{ opacity: 0, height: 0 }}
                transition={{ duration: 0.35, ease: [0.25, 1, 0.5, 1] }}
                className="flex items-start gap-3 text-[0.9rem] leading-relaxed overflow-hidden group/item"
                style={{ color: 'var(--text-secondary)' }}
              >
                <CheckCircle2
                  className="w-4 h-4 shrink-0 mt-0.5 transition-all duration-300 group-hover/item:scale-110"
                  style={{ color: accentColor, opacity: 0.75 }}
                />
                <span className="transition-colors duration-300 group-hover/item:text-[var(--text-primary)]">
                  {item}
                </span>
              </motion.li>
            ))}
        </AnimatePresence>
      </ul>

      {hidden.length > 0 && (
        <button
          type="button"
          onClick={() => setExpanded((v) => !v)}
          className="flex items-center gap-1.5 mt-4 text-xs font-bold uppercase tracking-wider transition-colors duration-300"
          style={{ color: accentColor }}
          aria-expanded={expanded}
        >
          {expanded ? 'Show less' : `Show ${hidden.length} more`}
          <ChevronDown
            className="w-3.5 h-3.5 transition-transform duration-300"
            style={{ transform: expanded ? 'rotate(180deg)' : 'none' }}
          />
        </button>
      )}
    </>
  );
}

'use client';

import { motion } from 'framer-motion';

export default function ProjectFilterTabs({
  filters,
  active,
  onChange,
}: {
  filters: string[];
  active: string;
  onChange: (filter: string) => void;
}) {
  return (
    <div
      className="flex flex-wrap items-center gap-1 p-1 rounded-full w-fit"
      style={{ background: 'var(--surface-elevated)', border: '1px solid var(--border-strong)' }}
      role="tablist"
      aria-label="Filter projects by category"
    >
      {filters.map((filter) => {
        const isActive = filter === active;
        return (
          <button
            key={filter}
            type="button"
            role="tab"
            aria-selected={isActive}
            onClick={() => onChange(filter)}
            className="relative px-4 py-2 rounded-full text-sm font-semibold transition-colors duration-300 outline-none"
            style={{ color: isActive ? '#ffffff' : 'var(--text-secondary)' }}
          >
            {isActive && (
              <motion.div
                layoutId="project-filter-pill"
                className="absolute inset-0 rounded-full -z-10"
                style={{ background: 'linear-gradient(135deg, var(--gradient-from), var(--gradient-via))' }}
                transition={{ type: 'spring', stiffness: 500, damping: 32 }}
              />
            )}
            <span className="relative z-10">{filter}</span>
          </button>
        );
      })}
    </div>
  );
}

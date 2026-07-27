'use client';

import { motion } from 'framer-motion';

/* ── Data ── */
const row1 = [
  'JavaScript (ES6+)', 'TypeScript', 'React 18', 'Next.js 15', 'Redux Toolkit',
  'Tailwind CSS v4', 'Framer Motion', 'Radix UI', 'Material UI', 'CSS3 / Sass', 'HTML5',
];

const row2 = [
  'Node.js', 'Express.js', 'MongoDB', 'Mongoose', 'PostgreSQL', 'Prisma ORM',
  'REST APIs', 'GraphQL', 'Git', 'GitHub Actions', 'Docker', 'Vercel', 'Vite', 'Jest', 'Figma',
];

const stackCategories = [
  {
    title: 'Frontend Architecture',
    description: 'Robust, scalable, pixel-perfect user interfaces.',
    number: '01',
    skills: ['JavaScript (ES6+)', 'TypeScript', 'React 18', 'Next.js 14/15', 'Redux Toolkit', 'Tailwind CSS v4', 'Material UI', 'Framer Motion', 'Radix UI', 'CSS3/Sass'],
  },
  {
    title: 'Backend Integration',
    description: 'Resilient server-side logic and scalable APIs.',
    number: '02',
    skills: ['Node.js', 'Express.js', 'REST APIs', 'Next.js API Routes', 'GraphQL Basics'],
  },
  {
    title: 'Database Management',
    description: 'Structured data modelling and high-performance persistence.',
    number: '03',
    skills: ['MongoDB', 'Mongoose', 'PostgreSQL', 'Prisma ORM'],
  },
  {
    title: 'DevOps & Tooling',
    description: 'Modern workflow optimisation, CI/CD, and deployments.',
    number: '04',
    skills: ['Git', 'GitHub Actions', 'Docker', 'Vercel', 'Vite', 'Postman', 'Figma', 'Jest'],
  },
];

/* Double arrays for seamless infinite loop */
const r1 = [...row1, ...row1];
const r2 = [...row2, ...row2];

/* ── Marquee pill ── */
function SkillPill({ label }: { label: string }) {
  return (
    <span className="skill-pill">
      <span
        className="w-1.5 h-1.5 rounded-full shrink-0"
        style={{ background: 'var(--accent)', opacity: 0.6 }}
      />
      {label}
    </span>
  );
}

/* ── Container variants for stagger ── */
const containerVariants = {
  hidden: {},
  visible: { transition: { staggerChildren: 0.12 } },
};

const cardVariants = {
  hidden:  { opacity: 0, y: 28, scale: 0.97 },
  visible: { opacity: 1, y: 0,  scale: 1,    transition: { duration: 0.65, ease: [0.25, 1, 0.5, 1] } },
};

export default function Stack() {
  return (
    <section id="stack" className="py-24 md:py-36 relative z-10 overflow-hidden">

      {/* ── Section header ── */}
      <div className="px-6 md:px-8">
        <div className="max-w-6xl mx-auto mb-20 text-center">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true, margin: '-100px' }}
            transition={{ duration: 0.8, ease: [0.25, 1, 0.5, 1] }}
          >
            <span className="section-label justify-center">Technologies</span>
            <h2 className="section-title text-5xl md:text-6xl lg:text-7xl mb-5 tracking-tighter">
              Tech <span className="text-accent">stack.</span>
            </h2>
            <p className="section-subtitle font-light md:text-xl mx-auto">
              My daily-driven toolkit for crafting performant, scalable modern web applications.
            </p>
          </motion.div>
        </div>
      </div>

      {/* ── Marquee rows ── */}
      <motion.div
        initial={{ opacity: 0 }}
        whileInView={{ opacity: 1 }}
        viewport={{ once: true }}
        transition={{ duration: 0.8, delay: 0.2 }}
        className="space-y-3 mb-20 flex flex-col items-center"
      >
        {/* Row 1 — scrolls left */}
        <div className="marquee-container py-1">
          <div className="marquee-track">
            {r1.map((skill, i) => (
              <SkillPill key={`r1-${i}`} label={skill} />
            ))}
          </div>
        </div>

        {/* Row 2 — scrolls right */}
        <div className="marquee-container py-1">
          <div className="marquee-track-reverse">
            {r2.map((skill, i) => (
              <SkillPill key={`r2-${i}`} label={skill} />
            ))}
          </div>
        </div>
      </motion.div>

      {/* ── Category cards ── */}
      <div className="px-6 md:px-8">
        <div className="max-w-6xl mx-auto">
          <motion.div
            variants={containerVariants}
            initial="hidden"
            whileInView="visible"
            viewport={{ once: true, margin: '-60px' }}
            className="grid grid-cols-1 md:grid-cols-2 gap-6"
          >
            {stackCategories.map((cat) => (
              <motion.div
                key={cat.title}
                variants={cardVariants}
                className="card group p-8 md:p-10 relative overflow-hidden gradient-border-card"
              >
                {/* Number watermark */}
                <span
                  className="absolute top-4 right-6 text-7xl font-black select-none pointer-events-none transition-all duration-500 group-hover:opacity-20"
                  style={{ color: 'var(--accent)', opacity: 0.06, letterSpacing: '-0.05em', lineHeight: 1 }}
                >
                  {cat.number}
                </span>

                {/* Hover gradient fill */}
                <div className="absolute inset-0 bg-linear-to-br from-accent/5 to-transparent opacity-0 transition-opacity duration-500 group-hover:opacity-100 z-0 pointer-events-none" />

                <div className="relative z-10">
                  <h3
                    className="text-xl font-display font-bold mb-1.5 transition-colors duration-300 group-hover:text-accent tracking-tight"
                    style={{ color: 'var(--text-primary)' }}
                  >
                    {cat.title}
                  </h3>
                  <p
                    className="text-[0.875rem] mb-6 font-medium leading-relaxed"
                    style={{ color: 'var(--text-tertiary)' }}
                  >
                    {cat.description}
                  </p>

                  <div className="flex flex-wrap gap-2">
                    {cat.skills.map((skill, i) => (
                      <motion.span
                        key={skill}
                        className="skill-badge"
                        initial={{ opacity: 0, scale: 0.85 }}
                        whileInView={{ opacity: 1, scale: 1 }}
                        viewport={{ once: true }}
                        transition={{ duration: 0.3, delay: i * 0.04 }}
                        whileHover={{ scale: 1.06, y: -1 }}
                      >
                        {skill}
                      </motion.span>
                    ))}
                  </div>
                </div>
              </motion.div>
            ))}
          </motion.div>
        </div>
      </div>
    </section>
  );
}

'use client';

import { motion } from 'framer-motion';

const stackCategories = [
  {
    title: 'Frontend Architecture',
    description: 'Constructing robust, highly scalable, pixel-perfect user interfaces.',
    skills: [
      'JavaScript (ES6+)',
      'TypeScript',
      'React 18',
      'Next.js 14/15',
      'Redux Toolkit',
      'Tailwind CSS v4',
      'Material UI',
      'Framer Motion',
      'Radix UI',
      'CSS3/Sass',
    ],
  },
  {
    title: 'Backend Integration',
    description: 'Designing resilient server-side logic and scalable APIs.',
    skills: ['Node.js', 'Express.js', 'REST APIs', 'Next.js Api Routes', 'GraphQL Basics'],
  },
  {
    title: 'Database Management',
    description: 'Structured data modeling and high-performance persistence.',
    skills: ['MongoDB', 'Mongoose', 'PostgreSQL', 'Prisma ORM'],
  },
  {
    title: 'DevOps & Tooling',
    description: 'Modern workflow optimization, CI/CD, and effortless deployments.',
    skills: ['Git', 'GitHub Actions', 'Docker', 'Vercel', 'Vite', 'Postman', 'Figma', 'Jest'],
  },
];

export default function Stack() {
  return (
    <section id="stack" className="py-24 md:py-36 px-6 md:px-8 relative z-10">
      <div className="max-w-6xl mx-auto relative z-10">
        <motion.div
          initial={{ opacity: 0, y: 24 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true, margin: '-100px' }}
          transition={{ duration: 0.8, ease: [0.25, 1, 0.5, 1] }}
          className="mb-20 text-center"
        >
          <span className="section-label group shadow-sm">
            <span className="inline-block w-2h-2 rounded-full bg-accent mr-3 animate-pulse"></span>
            Technologies
          </span>
          <h2 className="section-title text-5xl md:text-6xl lg:text-7xl mb-6 tracking-tighter">
            Tech <span className="text-accent">stack.</span>
          </h2>
          <p className="section-subtitle font-light md:text-xl mx-auto">
            My daily-driven toolkit for crafting performant, scalable modern web applications.
          </p>
        </motion.div>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-8 md:gap-10">
          {stackCategories.map((category, catIndex) => (
            <motion.div
              key={category.title}
              initial={{ opacity: 0, y: 30, scale: 0.98 }}
              whileInView={{ opacity: 1, y: 0, scale: 1 }}
              viewport={{ once: true, margin: '-50px' }}
              transition={{ duration: 0.7, delay: catIndex * 0.15, ease: [0.25, 1, 0.5, 1] }}
              className="card group p-8 md:p-10 relative overflow-hidden"
            >
              {/* Subtle hover gradient */}
              <div className="absolute inset-0 bg-linear-to-br from-accent/5 to-transparent opacity-0 transition-opacity duration-500 group-hover:opacity-100 z-0"></div>

              <div className="mb-6 relative z-10">
                <h3
                  className="text-2xl font-display font-bold mb-2 group-hover:text-accent transition-colors duration-300"
                  style={{ color: 'var(--text-primary)', letterSpacing: '-0.02em' }}
                >
                  {category.title}
                </h3>
                <p
                  className="text-[0.95rem] font-medium leading-relaxed"
                  style={{ color: 'var(--text-tertiary)' }}
                >
                  {category.description}
                </p>
              </div>
              <div className="flex flex-wrap gap-2.5 relative z-10">
                {category.skills.map((skill, i) => (
                  <motion.span 
                    key={skill} 
                    className="skill-badge hover:shadow-lg backdrop-blur-md"
                    whileHover={{ scale: 1.05 }}
                    transition={{ type: 'spring', stiffness: 400, damping: 10 }}
                  >
                    {skill}
                  </motion.span>
                ))}
              </div>
            </motion.div>
          ))}
        </div>
      </div>
    </section>
  );
}

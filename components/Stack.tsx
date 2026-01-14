'use client';

import { motion } from 'framer-motion';

const stackCategories = [
  {
    title: 'Frontend',
    skills: ['JavaScript', 'TypeScript', 'React', 'Next.js', 'Redux', 'Tailwind CSS', 'Material UI', 'Framer Motion']
  },
  {
    title: 'Backend',
    skills: ['Node.js', 'Express.js', 'REST APIs', 'Next.js API Routes']
  },
  {
    title: 'Database',
    skills: ['MongoDB', 'Mongoose', 'PostgreSQL']
  },
  {
    title: 'Tools',
    skills: ['Git', 'GitHub', 'Docker', 'Vercel', 'VS Code', 'Postman', 'Figma']
  }
];

export default function Stack() {
  return (
    <section id="stack" className="py-32 px-6 md:px-16 lg:px-24 bg-linear-to-b from-transparent via-black/30 to-transparent">
      <div className="max-w-6xl mx-auto">
        <motion.div
          initial={{ opacity: 0, y: 30 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.6 }}
          className="mb-16"
        >
          <span className="text-gray-500 text-sm uppercase tracking-widest mb-4 block">Technologies</span>
          <h2 className="font-display text-5xl md:text-6xl lg:text-7xl text-white">
            My Stack
          </h2>
        </motion.div>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-12">
          {stackCategories.map((category, catIndex) => (
            <motion.div
              key={category.title}
              initial={{ opacity: 0, y: 30 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ duration: 0.5, delay: catIndex * 0.1 }}
            >
              <h3 className="text-xl font-semibold text-white mb-6 flex items-center gap-3">
                <span className="w-8 h-px bg-white/30" />
                {category.title}
              </h3>
              <div className="flex flex-wrap gap-3">
                {category.skills.map((skill) => (
                  <span
                    key={skill}
                    className="skill-badge px-4 py-2 rounded-full text-sm text-gray-300 cursor-default"
                  >
                    {skill}
                  </span>
                ))}
              </div>
            </motion.div>
          ))}
        </div>
      </div>
    </section>
  );
}

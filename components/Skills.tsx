'use client';

import { motion } from 'framer-motion';
import { Layout, Server, Wrench } from 'lucide-react';

const skills = [
  {
    category: "Frontend Development",
    icon: <Layout className="w-6 h-6 text-blue-400" />,
    items: ["React", "Next.js", "TypeScript", "Tailwind CSS", "Framer Motion", "React Query", "Redux", "HTML5", "CSS3"]
  },
  {
    category: "Backend Development",
    icon: <Server className="w-6 h-6 text-green-400" />,
    items: ["Node.js", "Express", "MongoDB", "Mongoose", "REST APIs", "Next.js API Routes", "Axios", "Sequelize", "PostgreSQL"]
  },
  {
    category: "Tools & DevOps",
    icon: <Wrench className="w-6 h-6 text-purple-400" />,
    items: ["Git", "GitHub", "Docker", "VS Code", "Postman", "Vercel", "Figma", "npm"]
  }
];

export default function Skills() {
  return (
    <section id="skills" className="py-20 relative">
      <div className="container mx-auto px-6">
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.6 }}
          className="text-center mb-16"
        >
          <h2 className="text-4xl font-bold mb-4">Technical <span className="text-gradient">Skills</span></h2>
          <p className="text-gray-400 max-w-2xl mx-auto">
            A comprehensive toolset that allows me to build high-quality, scalable applications from concept to deployment.
          </p>
        </motion.div>

        <div className="grid grid-cols-1 md:grid-cols-3 gap-8">
          {skills.map((skillGroup, groupIndex) => (
            <motion.div
              key={skillGroup.category}
              initial={{ opacity: 0, y: 30 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ duration: 0.5, delay: groupIndex * 0.2 }}
              className="glass p-8 rounded-2xl hover:bg-white/5 transition-colors border border-white/5"
            >
              <div className="flex items-center gap-3 mb-6">
                <div className="p-3 rounded-lg bg-white/5">
                  {skillGroup.icon}
                </div>
                <h3 className="text-xl font-bold">{skillGroup.category}</h3>
              </div>
              
              <div className="flex flex-wrap gap-2">
                {skillGroup.items.map((item) => (
                  <span
                    key={item}
                    className={`px-3 py-1 text-sm bg-white/5 border border-white/10 rounded-full text-gray-300 hover:text-white hover:border-white/30 transition-all cursor-default ${
                      item === 'React Query' || item === 'Axios' 
                        ? 'bg-linear-to-r from-indigo-500/20 to-purple-500/20 border-purple-500/30 text-purple-300' 
                        : ''
                    }`}
                  >
                    {item}
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

'use client';

import { useState } from 'react';
import { motion } from 'framer-motion';
import { Github, ExternalLink, Star } from 'lucide-react';
import Image from 'next/image';

const projects = [
  {
    id: 1,
    title: "Talabat ElBeit",
    description: "A full-stack food delivery application built with TypeScript. Features include menu browsing, cart management, order tracking, and a complete backend API.",
    tags: ["TypeScript", "React", "Node.js", "Full-Stack"],
    githubLink: "https://github.com/Mohamed0Hussein/Talabat_ElBeit",
    demoLink: null,
    image: "https://images.unsplash.com/photo-1565299624946-b28f40a0ae38?w=800&q=80",
    featured: true
  },
  {
    id: 2,
    title: "Stories App",
    description: "A social media inspired stories application with TypeScript. Users can create, view, and interact with ephemeral story content similar to Instagram stories.",
    tags: ["TypeScript", "React", "Vercel", "Next.js"],
    githubLink: "https://github.com/Mohamed0Hussein/stories-app",
    demoLink: "https://stories-app-two.vercel.app",
    image: "https://images.unsplash.com/photo-1611162616305-c69b3fa7fbe0?w=800&q=80",
    featured: true
  },
  {
    id: 3,
    title: "Login Form",
    description: "A modern, secure authentication form with beautiful UI/UX. Features include form validation, responsive design, and smooth animations.",
    tags: ["TypeScript", "React", "Tailwind CSS", "Authentication"],
    githubLink: "https://github.com/Mohamed0Hussein/login-form",
    demoLink: "https://login-form-five-blue.vercel.app",
    image: "https://images.unsplash.com/photo-1555421689-491a97ff2040?w=800&q=80",
    featured: true
  },
  {
    id: 4,
    title: "Space Tourism",
    description: "An immersive space tourism website with beautiful animations and responsive design. Explores destinations, crew, and technology for space travel.",
    tags: ["TypeScript", "React", "CSS Animations", "Responsive"],
    githubLink: "https://github.com/Mohamed0Hussein/Space-tourism",
    demoLink: null,
    image: "https://images.unsplash.com/photo-1446776811953-b23d57bd21aa?w=800&q=80",
    featured: false
  },
  {
    id: 5,
    title: "Age Calculator",
    description: "A sleek age calculator application that computes exact years, months, and days between a date and today. Built with vanilla JavaScript.",
    tags: ["JavaScript", "CSS", "HTML", "Vercel"],
    githubLink: "https://github.com/Mohamed0Hussein/age-calculator",
    demoLink: "https://age-calculator-sand-rho.vercel.app",
    image: "https://images.unsplash.com/photo-1501139083538-0139583c060f?w=800&q=80",
    featured: false
  },
  {
    id: 6,
    title: "Flags API",
    description: "A REST API for country flags built with C#. Provides endpoints to fetch country information and flag images for integration into other applications.",
    tags: ["C#", "REST API", ".NET", "Backend"],
    githubLink: "https://github.com/Mohamed0Hussein/flags-api",
    demoLink: null,
    image: "https://images.unsplash.com/photo-1526778548025-fa2f459cd5c1?w=800&q=80",
    featured: false
  }
];

export default function Projects() {
  const [filter, setFilter] = useState<'all' | 'featured'>('all');
  
  const displayedProjects = filter === 'featured' 
    ? projects.filter(p => p.featured)
    : projects;

  return (
    <section id="projects" className="py-20 bg-black/20">
      <div className="container mx-auto px-6">
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.6 }}
          className="text-center mb-12"
        >
          <h2 className="text-4xl font-bold mb-4">Featured <span className="text-gradient">Projects</span></h2>
          <p className="text-gray-400 max-w-2xl mx-auto mb-8">
            A selection of projects that demonstrate my expertise in full-stack development.
          </p>

          <div className="flex justify-center gap-4">
            <button
              onClick={() => setFilter('all')}
              className={`px-6 py-2 rounded-full text-sm font-medium transition-all ${
                filter === 'all' 
                  ? 'bg-white text-black' 
                  : 'glass text-gray-300 hover:text-white border border-white/10'
              }`}
            >
              All Projects
            </button>
            <button
              onClick={() => setFilter('featured')}
              className={`px-6 py-2 rounded-full text-sm font-medium transition-all ${
                filter === 'featured' 
                  ? 'bg-white text-black' 
                  : 'glass text-gray-300 hover:text-white border border-white/10'
              }`}
            >
              Featured Only
            </button>
          </div>
        </motion.div>

        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8">
          {displayedProjects.map((project, index) => (
            <motion.div
              key={project.id}
              initial={{ opacity: 0, y: 30 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ duration: 0.5, delay: index * 0.1 }}
              className="group glass rounded-2xl overflow-hidden border border-white/5 hover:border-purple-500/30 transition-all duration-300"
            >
              <div className="h-48 w-full bg-linear-to-br from-gray-800 to-gray-900 relative overflow-hidden">
                <Image 
                  src={project.image} 
                  alt={project.title} 
                  fill
                  sizes="(max-width: 768px) 100vw, (max-width: 1200px) 50vw, 33vw"
                  className="object-cover transition-transform duration-500 group-hover:scale-110"
                />
                <div className="absolute inset-0 bg-black/50 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center gap-4">
                  {project.githubLink && (
                    <a 
                      href={project.githubLink} 
                      target="_blank" 
                      rel="noopener noreferrer"
                      className="p-3 bg-white/10 rounded-full hover:bg-white/20 hover:scale-110 transition-all backdrop-blur-sm"
                    >
                      <Github size={20} className="text-white" />
                    </a>
                  )}
                  {project.demoLink && (
                    <a 
                      href={project.demoLink} 
                      target="_blank" 
                      rel="noopener noreferrer"
                      className="p-3 bg-white/10 rounded-full hover:bg-white/20 hover:scale-110 transition-all backdrop-blur-sm"
                    >
                      <ExternalLink size={20} className="text-white" />
                    </a>
                  )}
                </div>
                
                {project.featured && (
                  <div className="absolute top-3 right-3 px-3 py-1 bg-linear-to-r from-indigo-500 to-purple-500 rounded-full text-xs font-semibold text-white flex items-center gap-1">
                    <Star size={12} /> Featured
                  </div>
                )}
              </div>

              <div className="p-6">
                <h3 className="text-xl font-bold mb-3">{project.title}</h3>
                <p className="text-gray-400 text-sm mb-6 line-clamp-3">
                  {project.description}
                </p>
                
                <div className="flex flex-wrap gap-2">
                  {project.tags.map(tag => (
                    <span key={tag} className="text-xs px-2 py-1 rounded-md bg-white/5 text-gray-300 border border-white/5">
                      {tag}
                    </span>
                  ))}
                </div>
              </div>
            </motion.div>
          ))}
        </div>

        <motion.div
          initial={{ opacity: 0 }}
          whileInView={{ opacity: 1 }}
          viewport={{ once: true }}
          transition={{ delay: 0.5 }}
          className="text-center mt-12"
        >
          <a
            href="https://github.com/Mohamed0Hussein?tab=repositories"
            target="_blank"
            rel="noopener noreferrer"
            className="inline-flex items-center gap-2 px-6 py-3 rounded-full glass text-white font-medium hover:bg-white/10 transition-colors border border-white/10"
          >
            <Github size={20} />
            View All Projects on GitHub
          </a>
        </motion.div>
      </div>
    </section>
  );
}

'use client';

import { motion } from 'framer-motion';
import { ArrowRight, Code, Database, Globe, Sparkles } from 'lucide-react';

export default function Hero() {
  return (
    <section className="relative min-h-screen flex items-center justify-center pt-20 overflow-hidden">
      <div className="absolute top-0 left-0 w-full h-full overflow-hidden -z-10">
        <div className="absolute top-[20%] left-[10%] w-72 h-72 bg-purple-600/20 rounded-full blur-[100px] animate-pulse" />
        <div className="absolute bottom-[20%] right-[10%] w-96 h-96 bg-indigo-600/20 rounded-full blur-[100px] animate-pulse" style={{ animationDelay: '1s' }} />
        <div className="absolute top-[50%] left-[50%] -translate-x-1/2 -translate-y-1/2 w-[600px] h-[600px] bg-pink-500/10 rounded-full blur-[150px]" />
      </div>

      <div className="container mx-auto px-6 text-center z-10">
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.8 }}
        >
          <div className="inline-flex items-center px-4 py-2 mb-8 rounded-full glass border border-white/10 text-sm text-gray-300">
            <span className="flex h-2 w-2 rounded-full bg-green-500 mr-2 animate-pulse"></span>
            Available for new opportunities
          </div>
        </motion.div>

        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.8, delay: 0.1 }}
          className="mb-4"
        >
          <span className="text-xl md:text-2xl text-gray-400">Hello, I&apos;m</span>
        </motion.div>

        <motion.h1
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.8, delay: 0.15 }}
          className="text-5xl md:text-7xl lg:text-8xl font-bold mb-6 tracking-tight"
        >
          <span className="text-gradient">Mohamed Hussein</span>
        </motion.h1>

        <motion.h2
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.8, delay: 0.2 }}
          className="text-2xl md:text-4xl font-semibold text-gray-300 mb-6"
        >
          Full Stack Developer
        </motion.h2>

        <motion.p
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.8, delay: 0.25 }}
          className="text-lg text-gray-400 max-w-2xl mx-auto mb-10"
        >
          Crafting scalable, pixel-perfect, and performant web applications with modern technologies. 
          Specializing in <span className="text-indigo-400">React</span>, <span className="text-purple-400">Next.js</span>, <span className="text-green-400">Node.js</span>, and <span className="text-pink-400">MongoDB</span>.
        </motion.p>

        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.8, delay: 0.3 }}
          className="flex flex-col md:flex-row items-center justify-center gap-4"
        >
          <a
            href="#projects"
            className="px-8 py-4 rounded-full bg-white text-black font-semibold hover:bg-gray-200 transition-colors flex items-center shadow-[0_0_30px_rgba(255,255,255,0.2)]"
          >
            View My Work <ArrowRight className="ml-2 w-5 h-5" />
          </a>
          <a
            href="#contact"
            className="px-8 py-4 rounded-full glass text-white font-semibold hover:bg-white/10 transition-colors border-white/10 border"
          >
            Get In Touch
          </a>
        </motion.div>

        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          transition={{ delay: 1, duration: 1 }}
          className="mt-24 pt-10 border-t border-white/5 grid grid-cols-2 md:grid-cols-4 gap-8 max-w-4xl mx-auto"
        >
          <div className="flex flex-col items-center gap-2 text-gray-400 hover:text-white transition-colors group">
            <div className="p-4 rounded-xl glass border border-white/5 group-hover:border-blue-500/30 transition-colors">
              <Code className="w-8 h-8 text-blue-400" />
            </div>
            <span className="font-medium text-sm">Frontend Excellence</span>
          </div>
          <div className="flex flex-col items-center gap-2 text-gray-400 hover:text-white transition-colors group">
            <div className="p-4 rounded-xl glass border border-white/5 group-hover:border-green-500/30 transition-colors">
              <Database className="w-8 h-8 text-green-400" />
            </div>
            <span className="font-medium text-sm">Robust Backend</span>
          </div>
          <div className="flex flex-col items-center gap-2 text-gray-400 hover:text-white transition-colors group">
            <div className="p-4 rounded-xl glass border border-white/5 group-hover:border-purple-500/30 transition-colors">
              <Globe className="w-8 h-8 text-purple-400" />
            </div>
            <span className="font-medium text-sm">Global Scale</span>
          </div>
          <div className="flex flex-col items-center gap-2 text-gray-400 hover:text-white transition-colors group">
            <div className="p-4 rounded-xl glass border border-white/5 group-hover:border-pink-500/30 transition-colors">
              <Sparkles className="w-8 h-8 text-pink-400" />
            </div>
            <span className="font-medium text-sm">Creative Design</span>
          </div>
        </motion.div>
      </div>
    </section>
  );
}

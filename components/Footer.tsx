'use client';

import { Github, Linkedin, Mail } from 'lucide-react';

export default function Footer() {
  return (
    <footer className="py-8 border-t border-white/5 bg-black/40">
      <div className="container mx-auto px-6 flex flex-col md:flex-row justify-between items-center text-gray-400 text-sm">
        <div className="mb-4 md:mb-0">
          © {new Date().getFullYear()} Mohamed Hussein. All rights reserved.
        </div>
        
        <div className="flex items-center space-x-6">
          <a href="#" className="hover:text-white transition-colors">Privacy Policy</a>
          <a href="#" className="hover:text-white transition-colors">Terms of Service</a>
          <div className="flex items-center space-x-4 ml-6 pl-6 border-l border-white/10">
            <a href="https://github.com/Mohamed0Hussein" target="_blank" rel="noopener noreferrer" className="hover:text-white transition-colors"><Github size={18} /></a>
            <a href="https://www.linkedin.com/in/mohamed-hussein-0x1/" target="_blank" rel="noopener noreferrer" className="hover:text-white transition-colors"><Linkedin size={18} /></a>
            <a href="mailto:mohamed.hussein0x1@gmail.com" className="hover:text-white transition-colors"><Mail size={18} /></a>
          </div>
        </div>
      </div>
    </footer>
  );
}

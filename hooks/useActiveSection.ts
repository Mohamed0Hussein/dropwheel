'use client';

import { useState, useEffect } from 'react';

const SECTIONS = ['hero', 'about', 'projects', 'stack', 'experience', 'contact'];

export function useActiveSection() {
  const [activeSection, setActiveSection] = useState('hero');

  useEffect(() => {
    const observers = new Map<string, IntersectionObserver>();

    SECTIONS.forEach((id) => {
      const el = document.getElementById(id);
      if (!el) return;

      const observer = new IntersectionObserver(
        (entries) => {
          entries.forEach((entry) => {
            if (entry.isIntersecting) setActiveSection(id);
          });
        },
        { rootMargin: '-35% 0px -55% 0px', threshold: 0 }
      );

      observer.observe(el);
      observers.set(id, observer);
    });

    return () => observers.forEach((obs) => obs.disconnect());
  }, []);

  return activeSection;
}

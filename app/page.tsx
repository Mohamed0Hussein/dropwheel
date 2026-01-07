import Navbar from '@/components/Navbar';
import Hero from '@/components/Hero';
import About from '@/components/About';
import Stack from '@/components/Stack';
import Experience from '@/components/Experience';
import Projects from '@/components/Projects';
import Contact from '@/components/Contact';
import SideElements from '@/components/SideElements';

export default function Home() {
  return (
    <main className="min-h-screen relative">
      <SideElements />
      <Navbar />
      <Hero />
      <About />
      <Stack />
      <Experience />
      <Projects />
      <Contact />
    </main>
  );
}

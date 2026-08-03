import Navbar from "@/components/Navbar";
import Hero from "@/components/Hero";
import About from "@/components/About";
import Stack from "@/components/Stack";
import Experience from "@/components/Experience";
import Projects from "@/components/Projects";
import Contact from "@/components/Contact";
import SideElements from "@/components/SideElements";
import Footer from "@/components/Footer";

export default function Home() {
  return (
    <main id="main-content" className="min-h-screen relative">
      <Navbar />
      <SideElements />
      <Hero />
      <About />
      <Projects />
      <Stack />
      <Experience />
      <Contact />
      <Footer />
    </main>
  );
}

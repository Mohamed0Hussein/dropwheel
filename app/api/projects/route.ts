import { NextResponse } from 'next/server';
import connectToDatabase from '@/lib/db';
import Project from '@/models/Project';

export async function GET() {
  try {
    await connectToDatabase();
    
    let projects = await Project.find({}).sort({ createdAt: -1 });

    // Internal seed if empty, for demonstration purposes so the portfolio isn't empty on first load without manual DB entry
    if (projects.length === 0) {
        const seedProjects = [
            {
                title: "E-Commerce Platform",
                description: "A full-stack e-commerce solution with real-time inventory management, stripe integration, and an admin dashboard.",
                tags: ["Next.js", "Node.js", "MongoDB", "Stripe"],
                githubLink: "#",
                demoLink: "#",
                image: "https://images.unsplash.com/photo-1557821552-17105176677c?w=800&q=80",
                featured: true
            },
            {
                title: "Task Management App",
                description: "Collaborative task manager with real-time updates using Socket.io and a drag-and-drop interface.",
                tags: ["React", "Express", "Socket.io", "Tailwind"],
                githubLink: "#",
                demoLink: "#",
                image: "https://images.unsplash.com/photo-1540350394557-8d14678e7f91?w=800&q=80",
                featured: true
            },
            {
                title: "AI Image Generator",
                description: "SaaS application that uses OpenAI's DALL-E API to generate images based on user prompts.",
                tags: ["Next.js", "OpenAI API", "PostgreSQL", "Prisma"],
                githubLink: "#",
                demoLink: "#",
                image: "https://images.unsplash.com/photo-1617791160505-6f00504e3519?w=800&q=80",
                featured: false
            }
        ];
        
        await Project.insertMany(seedProjects);
        projects = await Project.find({}).sort({ createdAt: -1 });
    }

    return NextResponse.json({ success: true, data: projects });
  } catch (error) {
    return NextResponse.json({ success: false, error: (error as Error).message }, { status: 500 });
  }
}

export async function POST(req: Request) {
    try {
        await connectToDatabase();
        const body = await req.json();
        const project = await Project.create(body);
        return NextResponse.json({ success: true, data: project }, { status: 201 });
    } catch (error) {
        return NextResponse.json({ success: false, error: (error as Error).message }, { status: 400 });
    }
}

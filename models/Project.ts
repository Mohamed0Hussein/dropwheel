import mongoose, { Schema, model, models } from 'mongoose';

const ProjectSchema = new Schema({
  title: { type: String, required: true },
  description: { type: String, required: true },
  tags: [{ type: String }],
  githubLink: { type: String },
  demoLink: { type: String },
  image: { type: String },
  featured: { type: Boolean, default: false },
  createdAt: { type: Date, default: Date.now },
});

const Project = models.Project || model('Project', ProjectSchema);

export default Project;

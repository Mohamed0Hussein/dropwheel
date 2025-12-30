import axios from 'axios';

const api = axios.create({
  baseURL: '/api',
  timeout: 10000,
  headers: {
    'Content-Type': 'application/json',
  },
});

export interface GitHubStats {
  publicRepos: number;
  followers: number;
  following: number;
  avatarUrl: string;
  bio: string;
  location: string;
  name: string;
  hireable: boolean;
  createdAt: string;
}

export const fetchGitHubStats = async (): Promise<GitHubStats> => {
  const { data } = await api.get('/github');
  if (!data.success) {
    throw new Error(data.error || 'Failed to fetch GitHub stats');
  }
  return data.data;
};

export interface ContactFormData {
  name: string;
  email: string;
  message: string;
}

export const submitContactForm = async (formData: ContactFormData) => {
  const { data } = await api.post('/contact', formData);
  if (!data.success) {
    throw new Error(data.error || 'Failed to submit form');
  }
  return data.data;
};

export default api;

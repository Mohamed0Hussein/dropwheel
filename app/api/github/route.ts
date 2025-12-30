import { NextResponse } from 'next/server';

export async function GET() {
  try {
    const response = await fetch('https://api.github.com/users/Mohamed0Hussein', {
      headers: {
        'Accept': 'application/vnd.github.v3+json',
      },
      next: { revalidate: 3600 }
    });

    if (!response.ok) {
      throw new Error('Failed to fetch GitHub data');
    }

    const data = await response.json();

    return NextResponse.json({
      success: true,
      data: {
        publicRepos: data.public_repos,
        followers: data.followers,
        following: data.following,
        avatarUrl: data.avatar_url,
        bio: data.bio,
        location: data.location,
        name: data.name,
        hireable: data.hireable,
        createdAt: data.created_at,
      }
    });
  } catch (error) {
    return NextResponse.json(
      { success: false, error: (error as Error).message },
      { status: 500 }
    );
  }
}

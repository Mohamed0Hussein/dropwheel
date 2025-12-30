import { NextResponse } from 'next/server';
import connectToDatabase from '@/lib/db';
import Message from '@/models/Message';

export async function POST(req: Request) {
  try {
    await connectToDatabase();
    const body = await req.json();
    
    if (!body.name || !body.email || !body.message) {
        return NextResponse.json({ success: false, error: 'Missing required fields' }, { status: 400 });
    }

    const message = await Message.create(body);
    
    return NextResponse.json({ success: true, data: message }, { status: 201 });
  } catch (error) {
    return NextResponse.json({ success: false, error: (error as Error).message }, { status: 500 });
  }
}

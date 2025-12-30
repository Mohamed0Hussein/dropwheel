import mongoose, { Schema, model, models } from 'mongoose';

const MessageSchema = new Schema({
  name: { type: String, required: true },
  email: { type: String, required: true },
  message: { type: String, required: true },
  read: { type: Boolean, default: false },
  createdAt: { type: Date, default: Date.now },
});

const Message = models.Message || model('Message', MessageSchema);

export default Message;

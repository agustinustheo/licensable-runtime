import 'reflect-metadata';
import dotenv from 'dotenv';
import { NestFactory } from '@nestjs/core';
import { AppModule } from './app.module';

dotenv.config();

async function bootstrap() {
  const app = await NestFactory.create(AppModule);
  const port = process.env.PORT || 3000;

  await app.listen(port);
  console.log(`License API is running on http://localhost:${port}`);
  console.log(`GET /license?key=YOUR_KEY_HERE`);
}

bootstrap();

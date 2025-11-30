import { Module } from '@nestjs/common';
import { TypeOrmModule } from '@nestjs/typeorm';
import { License } from './entities/license.entity';
import { LicenseService } from './services/license.service';
import { LicenseController } from './controllers/license.controller';

@Module({
  imports: [
    TypeOrmModule.forRoot({
      type: 'postgres',
      host: process.env.DB_HOST || 'localhost',
      port: parseInt(process.env.DB_PORT || '5432'),
      username: process.env.DB_USERNAME || 'postgres',
      password: process.env.DB_PASSWORD || 'postgres',
      database: process.env.DB_NAME || 'license_db',
      entities: [License],
      synchronize: true,
      logging: process.env.NODE_ENV !== 'production',
    }),
    TypeOrmModule.forFeature([License]),
  ],
  controllers: [LicenseController],
  providers: [LicenseService],
})
export class AppModule {}

import 'reflect-metadata';
import dotenv from 'dotenv';
import { DataSource } from 'typeorm';
import { License } from './entities/license.entity';

dotenv.config();

const AppDataSource = new DataSource({
  type: 'postgres',
  host: process.env.DB_HOST || 'localhost',
  port: parseInt(process.env.DB_PORT || '5432'),
  username: process.env.DB_USERNAME || 'postgres',
  password: process.env.DB_PASSWORD || 'password',
  database: process.env.DB_NAME || 'license_db',
  entities: [License],
  synchronize: true,
  logging: true,
});

async function seed() {
  try {
    await AppDataSource.initialize();
    console.log('Database connection established');

    const licenseRepository = AppDataSource.getRepository(License);

    // Clear existing licenses
    await licenseRepository.delete({});

    // Create sample licenses
    const now = new Date();
    const futureDate = new Date(now.getTime() + 30 * 24 * 60 * 60 * 1000); // 30 days from now
    const pastDate = new Date(now.getTime() - 30 * 24 * 60 * 60 * 1000); // 30 days ago

    const validLicense = licenseRepository.create({
      key: 'valid-license-key-12345',
      expiresAt: futureDate,
      isActive: true,
    });

    const expiredLicense = licenseRepository.create({
      key: 'expired-license-key-67890',
      expiresAt: pastDate,
      isActive: true,
    });

    const inactiveLicense = licenseRepository.create({
      key: 'inactive-license-key-11111',
      expiresAt: futureDate,
      isActive: false,
    });

    await licenseRepository.save([validLicense, expiredLicense, inactiveLicense]);

    console.log('Seed data created successfully!');
    console.log('\nTest licenses:');
    console.log('Valid:   valid-license-key-12345');
    console.log('Expired: expired-license-key-67890');
    console.log('Inactive: inactive-license-key-11111');

    await AppDataSource.destroy();
  } catch (error) {
    console.error('Seed failed:', error);
    process.exit(1);
  }
}

seed();

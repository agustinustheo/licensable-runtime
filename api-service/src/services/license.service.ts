import { Injectable } from '@nestjs/common';
import { InjectRepository } from '@nestjs/typeorm';
import { Repository } from 'typeorm';
import { License } from '../entities/license.entity';

@Injectable()
export class LicenseService {
  constructor(
    @InjectRepository(License)
    private readonly licenseRepository: Repository<License>
  ) {}

  async validateLicense(key: string): Promise<boolean> {
    const license = await this.licenseRepository.findOne({
      where: { key },
    });

    if (!license) {
      return false;
    }

    // Check if license is active and not expired
    const now = new Date();
    return license.isActive && license.expiresAt > now;
  }
}

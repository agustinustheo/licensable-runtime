import { Controller, Get, Query, BadRequestException } from '@nestjs/common';
import { LicenseService } from '../services/license.service';

@Controller()
export class LicenseController {
  constructor(private readonly licenseService: LicenseService) {}

  @Get('health')
  async healthCheck() {
    return {
      status: 'ok',
      service: 'license-api',
      timestamp: new Date().toISOString()
    };
  }

  @Get('license')
  async validateLicense(@Query('key') key: string) {
    if (!key) {
      throw new BadRequestException('License key is required');
    }

    const isValid = await this.licenseService.validateLicense(key);
    return { valid: isValid };
  }
}

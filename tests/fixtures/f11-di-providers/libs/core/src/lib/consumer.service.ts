import { inject, Injectable, InjectionToken } from '@angular/core';
import { ApiService } from './api.service';
import { AppConfig } from './config';
import { FileLogger, Logger } from './logger';

export const APP_CONFIG = new InjectionToken<AppConfig>('app.config');

export const LOGGER_PROVIDERS = [{ provide: Logger, useClass: FileLogger }];

@Injectable({ providedIn: 'root' })
export class ConsumerService {
  private api = inject(ApiService);

  constructor(private logger: Logger) {}

  run(): string {
    this.logger.log('run');
    return this.api.fetch();
  }
}

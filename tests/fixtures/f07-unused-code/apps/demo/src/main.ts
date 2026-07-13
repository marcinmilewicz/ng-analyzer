import { mainOnlyUtil } from '@fix/stuff';
import type { FetchResult } from '@fix/stuff';
import { AppComponent } from './app/app.component';

export function bootstrap(): unknown {
  mainOnlyUtil();
  const result: FetchResult = { success: true };
  return result && AppComponent;
}

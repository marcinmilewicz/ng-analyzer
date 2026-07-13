import { CommonModule } from '@angular/common';
import { NgModule } from '@angular/core';
import { DetailComponent } from './detail.component';
import { LegacyStateService } from './legacy-state.service';
import { ListComponent } from './list.component';

@NgModule({
  declarations: [ListComponent, DetailComponent],
  imports: [CommonModule],
  exports: [ListComponent],
  providers: [LegacyStateService],
  bootstrap: [ListComponent],
})
export class LegacyModule {}

import { Component, EventEmitter, Input, Output, input, model, output } from '@angular/core';

@Component({
  selector: 'fix-badge',
  template: '<span class="badge">{{ label() }}</span>',
  styleUrl: './badge.component.css',
})
export class BadgeComponent {
  label = input.required<string>();
  variant = input<'info' | 'warn'>('info');
  counter = model(0);
  dismissed = output<void>();

  @Input() legacyTitle = '';
  @Output() legacyClosed = new EventEmitter<void>();
}

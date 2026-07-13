export const routes = [
  {
    path: 'lazy',
    loadChildren: () => import('@fix/feature-lazy').then((m) => m.lazyRoutes),
  },
  {
    path: 'page',
    loadComponent: () =>
      import('@fix/feature-page').then((m) => m.LazyPageComponent),
  },
];

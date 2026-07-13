import { lazy } from 'react';
import { Button, Card } from '@fix/react-ui';

const LazySettings = lazy(() => import('./settings'));

export function App(props: { title: string }): unknown {
  return (
    <Card elevated>
      <h1>{props.title}</h1>
      <Button variant="primary" size="lg" onClick={() => null}>
        Buy
      </Button>
      <Button variant="ghost">Cancel</Button>
      <LazySettings />
    </Card>
  );
}

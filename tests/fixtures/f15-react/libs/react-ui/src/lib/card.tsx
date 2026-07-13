import { memo } from 'react';

export const Card = memo((props: { elevated?: boolean; children?: unknown }) => {
  return <div className="card">{props.children}</div>;
});

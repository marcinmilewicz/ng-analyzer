import { WebPanel } from '@fix/web-ui';

export interface PageProps {
  params: Promise<Record<string, string>>;
}

export default function DashboardPage(props: PageProps): unknown {
  return <WebPanel title="dashboard" />;
}

export interface LayoutProps {
  children: unknown;
}

export const metadata = { title: 'f20' };

export default function RootLayout(props: LayoutProps): unknown {
  return <html><body>{props.children}</body></html>;
}

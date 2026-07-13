export function Button(props: {
  variant: string;
  size?: string;
  onClick?: () => void;
  children?: unknown;
}): unknown {
  return <button className={props.variant}>{props.children}</button>;
}

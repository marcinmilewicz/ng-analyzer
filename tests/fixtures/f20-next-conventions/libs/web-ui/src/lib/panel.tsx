export function WebPanel(props: { title: string }): unknown {
  return <section>{props.title}</section>;
}

export function TrulyDeadWidget(): unknown {
  return <em>dead</em>;
}

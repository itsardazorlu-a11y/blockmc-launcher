export default function Soon({ title, phase, text }: { title: string; phase: string; text: string }) {
  return (
    <>
      <div className="head"><h1>{title}</h1></div>
      <div className="empty"><h2>Not built yet</h2><p>{text} This page is planned for {phase}.</p></div>
    </>
  );
}

interface HexLogoProps {
  size?: number;
}

export function HexLogo({ size = 28 }: HexLogoProps) {
  return (
    <img
      src="/logo.png"
      alt="Vectorizer"
      width={size}
      height={size}
      style={{ display: 'block' }}
    />
  );
}

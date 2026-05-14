interface Props {
  message: string;
}

export default function ErrorBanner({ message }: Props) {
  return (
    <p className="text-sm text-red-400 bg-red-400/10 border border-red-400/20 rounded-lg px-4 py-3">
      {message}
    </p>
  );
}

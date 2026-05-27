/**
 * Full-page loading fallback shown while a lazily-loaded route chunk is
 * being fetched. Keeps the background consistent with the app shell.
 */
export default function PageLoader() {
  return (
    <div className="flex min-h-screen items-center justify-center bg-zinc-950">
      <div className="h-8 w-8 animate-spin rounded-full border-4 border-zinc-700 border-t-indigo-500" />
    </div>
  );
}

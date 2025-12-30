import Link from 'next/link';

export default function HomePage() {
  return (
    <div className="flex flex-col justify-center text-center flex-1 px-4">
      <h1 className="text-4xl font-bold mb-4">kdb_codec</h1>
      <p className="text-xl text-muted-foreground mb-8 max-w-2xl mx-auto">
        A Rust library for kdb+ IPC (Inter-Process Communication) with true cancellation safety,
        built on tokio-util codec pattern.
      </p>
      <div className="flex flex-wrap gap-4 justify-center mb-8">
        <Link
          href="/docs"
          className="px-6 py-3 bg-primary text-primary-foreground font-medium rounded-lg hover:bg-primary/90 transition-colors"
        >
          Get Started
        </Link>
        <a
          href="https://github.com/yshing/kdb_codec"
          className="px-6 py-3 border border-border font-medium rounded-lg hover:bg-accent transition-colors"
          target="_blank"
          rel="noopener noreferrer"
        >
          View on GitHub
        </a>
      </div>
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6 max-w-4xl mx-auto text-left">
        <div className="p-6 border border-border rounded-lg">
          <h3 className="font-semibold text-lg mb-2">✅ Cancellation Safe</h3>
          <p className="text-muted-foreground text-sm">
            Built on tokio-util codec pattern for true cancellation safety with tokio::select!
          </p>
        </div>
        <div className="p-6 border border-border rounded-lg">
          <h3 className="font-semibold text-lg mb-2">🦀 Idiomatic Rust</h3>
          <p className="text-muted-foreground text-sm">
            Type-safe API with Index traits for ergonomic data access using familiar [] syntax
          </p>
        </div>
        <div className="p-6 border border-border rounded-lg">
          <h3 className="font-semibold text-lg mb-2">🚀 Full Featured</h3>
          <p className="text-muted-foreground text-sm">
            TCP, TLS, Unix socket support with full compression compatibility
          </p>
        </div>
      </div>
    </div>
  );
}

import { Component, type ErrorInfo, type ReactNode } from 'react';

interface Props {
  children: ReactNode;
}

interface State {
  error: Error | null;
}

/**
 * Catches unhandled render errors in the component tree and shows a
 * recovery UI instead of a blank screen.
 *
 * React does not yet expose a hook-based error boundary API, so this must
 * remain a class component.
 */
export class ErrorBoundary extends Component<Props, State> {
  state: State = { error: null };

  static getDerivedStateFromError(error: Error): State {
    return { error };
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    console.error('[ErrorBoundary] Uncaught render error:', error, info.componentStack);
  }

  private handleReload = () => {
    window.location.reload();
  };

  render() {
    if (this.state.error) {
      return (
        <div className="flex min-h-screen items-center justify-center bg-zinc-950 p-6">
          <div className="max-w-md w-full rounded-2xl bg-zinc-900 border border-zinc-800 p-8 text-center space-y-4">
            <div className="text-4xl">⚠️</div>
            <h1 className="text-xl font-semibold text-white">Etwas ist schiefgelaufen</h1>
            <p className="text-sm text-zinc-400">
              Ein unerwarteter Fehler ist aufgetreten. Du kannst die Seite neu laden oder es später
              erneut versuchen.
            </p>
            <details className="text-left text-xs text-zinc-500 bg-zinc-800 rounded-lg p-3 max-h-32 overflow-auto">
              <summary className="cursor-pointer mb-1">Fehlerdetails</summary>
              {this.state.error.message}
            </details>
            <button
              onClick={this.handleReload}
              className="w-full rounded-xl bg-indigo-600 hover:bg-indigo-500 text-white font-medium py-2 px-4 transition-colors"
            >
              Seite neu laden
            </button>
          </div>
        </div>
      );
    }

    return this.props.children;
  }
}

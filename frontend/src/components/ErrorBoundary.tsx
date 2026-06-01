import { Component, type ErrorInfo, type ReactNode } from 'react';
import { withTranslation, type WithTranslation } from 'react-i18next';

interface Props extends WithTranslation {
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
 * remain a class component. i18n is injected via the `withTranslation` HOC.
 */
class ErrorBoundaryInner extends Component<Props, State> {
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
    const { t } = this.props;
    if (this.state.error) {
      return (
        <div className="flex min-h-screen items-center justify-center bg-app p-6">
          <div className="max-w-md w-full rounded-2xl bg-ui-surface border border-ui-border p-8 text-center space-y-4">
            <div className="text-4xl">⚠️</div>
            <h1 className="text-xl font-semibold text-white">{t('error_boundary.title')}</h1>
            <p className="text-sm text-zinc-400">
              {t('error_boundary.message')}
            </p>
            <details className="text-left text-xs text-zinc-500 bg-ui-raised rounded-lg p-3 max-h-32 overflow-auto">
              <summary className="cursor-pointer mb-1">{t('error_boundary.details_summary')}</summary>
              {this.state.error.message}
            </details>
            <button
              onClick={this.handleReload}
              className="w-full rounded-xl bg-indigo-600 hover:bg-indigo-500 text-white font-medium py-2 px-4 transition-colors"
            >
              {t('error_boundary.reload')}
            </button>
          </div>
        </div>
      );
    }

    return this.props.children;
  }
}

export const ErrorBoundary = withTranslation()(ErrorBoundaryInner);

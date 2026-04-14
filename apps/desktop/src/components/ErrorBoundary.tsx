import { Component, type ReactNode, type ErrorInfo } from "react";

interface Props {
  children: ReactNode;
}

interface State {
  hasError: boolean;
  error: Error | null;
  errorInfo: ErrorInfo | null;
}

export default class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = { hasError: false, error: null, errorInfo: null };
  }

  static getDerivedStateFromError(error: Error): Partial<State> {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    console.error("ErrorBoundary caught:", error, errorInfo);
    this.setState({ errorInfo });
  }

  render() {
    if (this.state.hasError) {
      return (
        <div className="min-h-screen p-8 bg-cyber-bg text-cyber-text">
          <div className="max-w-2xl mx-auto">
            <h1 className="text-2xl font-bold text-cyber-red mb-4">
              Something went wrong
            </h1>
            <div className="bg-cyber-card border border-cyber-red/50 rounded-lg p-4 mb-4">
              <p className="text-cyber-red font-mono text-sm mb-2">
                {this.state.error?.message}
              </p>
              <pre className="text-cyber-text-dim text-xs font-mono whitespace-pre-wrap overflow-auto max-h-64">
                {this.state.error?.stack}
              </pre>
            </div>
            {this.state.errorInfo && (
              <div className="bg-cyber-card border border-cyber-border rounded-lg p-4">
                <p className="text-cyber-text-dim text-xs font-mono mb-2">
                  Component stack:
                </p>
                <pre className="text-cyber-text-dim text-xs font-mono whitespace-pre-wrap overflow-auto max-h-64">
                  {this.state.errorInfo.componentStack}
                </pre>
              </div>
            )}
            <button
              onClick={() =>
                this.setState({
                  hasError: false,
                  error: null,
                  errorInfo: null,
                })
              }
              className="mt-4 px-4 py-2 bg-cyber-card border border-cyber-border hover:border-cyber-border-bright rounded text-sm"
            >
              Try Again
            </button>
          </div>
        </div>
      );
    }

    return this.props.children;
  }
}

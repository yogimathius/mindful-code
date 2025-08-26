import * as vscode from 'vscode';
import { SessionManager } from './SessionManager';

export class ActivityTracker {
  private disposables: vscode.Disposable[] = [];
  private lastActivityTime: Date = new Date();

  constructor(private sessionManager: SessionManager) {}

  activate(context: vscode.ExtensionContext): void {
    // Track text document changes (typing)
    const onDocumentChange = vscode.workspace.onDidChangeTextDocument((event) => {
      if (event.contentChanges.length > 0) {
        this.recordActivity(event.document.uri.fsPath);
      }
    });

    // Track active editor changes (file switching)
    const onActiveEditorChange = vscode.window.onDidChangeActiveTextEditor((editor) => {
      if (editor) {
        this.recordActivity(editor.document.uri.fsPath);
      }
    });

    // Track window focus changes
    const onWindowFocusChange = vscode.window.onDidChangeWindowState((state) => {
      if (state.focused) {
        this.recordActivity();
      }
    });

    // Track selection changes (navigation)
    const onSelectionChange = vscode.window.onDidChangeTextEditorSelection((event) => {
      this.recordActivity(event.textEditor.document.uri.fsPath);
    });

    // Track visible range changes (scrolling)
    const onVisibleRangesChange = vscode.window.onDidChangeTextEditorVisibleRanges((event) => {
      this.recordActivity(event.textEditor.document.uri.fsPath);
    });

    // Track terminal activity (if available in VS Code API version)
    let onTerminalWrite: vscode.Disposable | undefined;
    if ('onDidWriteTerminalData' in vscode.window) {
      onTerminalWrite = (vscode.window as any).onDidWriteTerminalData((event: any) => {
        this.recordActivity();
      });
    }

    // Track debug session changes
    const onDebugSessionStart = vscode.debug.onDidStartDebugSession(() => {
      this.recordActivity();
    });

    const onDebugSessionTerminate = vscode.debug.onDidTerminateDebugSession(() => {
      this.recordActivity();
    });

    // Store disposables
    this.disposables.push(
      onDocumentChange,
      onActiveEditorChange,
      onWindowFocusChange,
      onSelectionChange,
      onVisibleRangesChange,
      onDebugSessionStart,
      onDebugSessionTerminate
    );
    
    if (onTerminalWrite) {
      this.disposables.push(onTerminalWrite);
    }

    // Add to context subscriptions
    context.subscriptions.push(...this.disposables);

    console.log('ActivityTracker activated');
  }

  private recordActivity(file?: string): void {
    const now = new Date();
    
    // Only record activity if it's been at least 1 second since last activity
    // This prevents excessive recording during rapid typing
    if (now.getTime() - this.lastActivityTime.getTime() < 1000) {
      return;
    }

    this.lastActivityTime = now;
    
    // Filter out non-workspace files and temporary files
    if (file && this.shouldTrackFile(file)) {
      this.sessionManager.recordActivity(file);
    } else {
      this.sessionManager.recordActivity();
    }
  }

  private shouldTrackFile(filePath: string): boolean {
    // Don't track files outside workspace
    const workspaceFolders = vscode.workspace.workspaceFolders;
    if (!workspaceFolders || workspaceFolders.length === 0) {
      return false;
    }

    const isInWorkspace = workspaceFolders.some(folder => 
      filePath.startsWith(folder.uri.fsPath)
    );
    
    if (!isInWorkspace) {
      return false;
    }

    // Don't track certain file types
    const excludePatterns = [
      /node_modules/,
      /\.git/,
      /\.vscode/,
      /dist/,
      /build/,
      /coverage/,
      /\.log$/,
      /\.tmp$/,
      /\.temp$/,
    ];

    return !excludePatterns.some(pattern => pattern.test(filePath));
  }

  getLastActivityTime(): Date {
    return this.lastActivityTime;
  }

  dispose(): void {
    this.disposables.forEach(disposable => disposable.dispose());
    this.disposables = [];
  }
}
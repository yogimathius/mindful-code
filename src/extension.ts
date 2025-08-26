import * as vscode from 'vscode';
import { SessionManager } from './services/SessionManager';
import { ActivityTracker } from './services/ActivityTracker';

let sessionManager: SessionManager;
let activityTracker: ActivityTracker;

export function activate(context: vscode.ExtensionContext): void {
  console.log('Mindful Code extension activated');

  // Initialize services
  sessionManager = new SessionManager(context);
  activityTracker = new ActivityTracker(sessionManager);

  // Register commands
  const startSessionCommand = vscode.commands.registerCommand(
    'mindfulCode.startSession',
    () => {
      sessionManager.startSession();
      vscode.window.showInformationMessage('Coding session started!');
    }
  );

  const pauseSessionCommand = vscode.commands.registerCommand(
    'mindfulCode.pauseSession',
    () => {
      sessionManager.pauseSession();
      vscode.window.showInformationMessage('Session paused');
    }
  );

  const endSessionCommand = vscode.commands.registerCommand(
    'mindfulCode.endSession',
    async () => {
      await sessionManager.endSession();
      // Notification is now handled by NotificationService
    }
  );

  const showDashboardCommand = vscode.commands.registerCommand(
    'mindfulCode.showDashboard',
    () => {
      // TODO: Implement dashboard webview
      vscode.window.showInformationMessage('Dashboard coming soon!');
    }
  );

  // Register event listeners
  activityTracker.activate(context);

  // Add disposables to context
  context.subscriptions.push(
    startSessionCommand,
    pauseSessionCommand,
    endSessionCommand,
    showDashboardCommand
  );

  // Auto-start session if configured
  const config = vscode.workspace.getConfiguration('mindfulCode');
  const autoStart = config.get<boolean>('autoStartSession', false);
  if (autoStart) {
    sessionManager.startSession();
  }
}

export function deactivate(): void {
  console.log('Mindful Code extension deactivated');
  if (sessionManager) {
    sessionManager.endSession();
  }
  if (activityTracker) {
    activityTracker.dispose();
  }
}
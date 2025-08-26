import * as vscode from 'vscode';
import { Session, SessionData } from '../models/Session';
import { DatabaseService } from './DatabaseService';

export class SessionManager {
  private currentSession: Session | null = null;
  private database: DatabaseService;
  private statusBarItem: vscode.StatusBarItem;
  private updateTimer: NodeJS.Timeout | null = null;
  private autoSaveTimer: NodeJS.Timeout | null = null;

  constructor(private context: vscode.ExtensionContext) {
    this.database = new DatabaseService(context.globalStorageUri.fsPath);
    
    // Create status bar item
    this.statusBarItem = vscode.window.createStatusBarItem(
      vscode.StatusBarAlignment.Left,
      100
    );
    this.statusBarItem.command = 'mindfulCode.showDashboard';
    this.context.subscriptions.push(this.statusBarItem);

    this.updateStatusBar();
  }

  async startSession(): Promise<void> {
    if (this.currentSession?.isActive) {
      vscode.window.showWarningMessage('Session already active');
      return;
    }

    if (this.currentSession?.isPaused) {
      this.resumeSession();
      return;
    }

    this.currentSession = new Session();
    this.currentSession.start();
    
    await this.database.saveSession(this.currentSession.toJSON());
    
    this.startTimers();
    this.updateStatusBar();
    
    console.log(`Session started: ${this.currentSession.id}`);
  }

  pauseSession(): void {
    if (!this.currentSession?.isActive) {
      vscode.window.showWarningMessage('No active session to pause');
      return;
    }

    this.currentSession.pause();
    this.stopTimers();
    this.updateStatusBar();
    
    console.log(`Session paused: ${this.currentSession.id}`);
  }

  resumeSession(): void {
    if (!this.currentSession?.isPaused) {
      vscode.window.showWarningMessage('No paused session to resume');
      return;
    }

    this.currentSession.resume();
    this.startTimers();
    this.updateStatusBar();
    
    console.log(`Session resumed: ${this.currentSession.id}`);
  }

  endSession(): SessionData | null {
    if (!this.currentSession) {
      vscode.window.showWarningMessage('No session to end');
      return null;
    }

    this.currentSession.end();
    const sessionData = this.currentSession.toJSON();
    
    this.database.saveSession(sessionData);
    this.stopTimers();
    
    const result = sessionData;
    this.currentSession = null;
    this.updateStatusBar();
    
    console.log(`Session ended: ${result.id}, Duration: ${result.duration}ms`);
    return result;
  }

  recordActivity(file?: string): void {
    if (this.currentSession) {
      this.currentSession.recordActivity(file);
      this.checkAutoResume();
    }
  }

  getCurrentSession(): Session | null {
    return this.currentSession;
  }

  async getSessionHistory(days: number = 30): Promise<SessionData[]> {
    return this.database.getRecentSessions(days);
  }

  private startTimers(): void {
    // Update status bar every 5 seconds
    this.updateTimer = setInterval(() => {
      this.updateStatusBar();
      this.checkIdleTimeout();
    }, 5000);

    // Auto-save every 30 seconds
    this.autoSaveTimer = setInterval(() => {
      if (this.currentSession) {
        this.database.saveSession(this.currentSession.toJSON());
      }
    }, 30000);
  }

  private stopTimers(): void {
    if (this.updateTimer) {
      clearInterval(this.updateTimer);
      this.updateTimer = null;
    }
    if (this.autoSaveTimer) {
      clearInterval(this.autoSaveTimer);
      this.autoSaveTimer = null;
    }
  }

  private checkIdleTimeout(): void {
    if (!this.currentSession) {
      return;
    }

    const config = vscode.workspace.getConfiguration('mindfulCode');
    const idleTimeoutMinutes = config.get<number>('idleTimeoutMinutes', 5);
    const idleTimeoutMs = idleTimeoutMinutes * 60 * 1000;

    if (this.currentSession.shouldAutoPause(idleTimeoutMs)) {
      this.pauseSession();
      
      const showNotifications = config.get<boolean>('showNotifications', true);
      if (showNotifications) {
        vscode.window.showInformationMessage(
          `Session auto-paused after ${idleTimeoutMinutes} minutes of inactivity`
        );
      }
    }
  }

  private checkAutoResume(): void {
    if (this.currentSession?.isPaused) {
      this.resumeSession();
      
      const config = vscode.workspace.getConfiguration('mindfulCode');
      const showNotifications = config.get<boolean>('showNotifications', true);
      if (showNotifications) {
        vscode.window.showInformationMessage('Session auto-resumed');
      }
    }
  }

  private updateStatusBar(): void {
    if (!this.currentSession) {
      this.statusBarItem.text = '$(play) Start Session';
      this.statusBarItem.tooltip = 'Click to start a coding session';
      this.statusBarItem.show();
      return;
    }

    const session = this.currentSession;
    const duration = Math.round(session.toJSON().duration / 1000 / 60);
    
    if (session.isPaused) {
      this.statusBarItem.text = `$(debug-pause) ${duration}m (paused)`;
      this.statusBarItem.tooltip = 'Session paused - Click to view dashboard';
    } else if (session.isActive) {
      this.statusBarItem.text = `$(pulse) ${duration}m`;
      this.statusBarItem.tooltip = `Active session: ${duration} minutes - Click to view dashboard`;
    }
    
    this.statusBarItem.show();
  }

  dispose(): void {
    this.stopTimers();
    if (this.currentSession) {
      this.endSession();
    }
    this.statusBarItem.dispose();
  }
}
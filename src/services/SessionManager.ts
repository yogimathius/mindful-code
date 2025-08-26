import * as vscode from 'vscode';
import { Session, SessionData } from '../models/Session';
import { DatabaseService } from './DatabaseService';
import { NotificationService } from './NotificationService';
import { FlowStateAnalyzer } from './FlowStateAnalyzer';

export class SessionManager {
  private currentSession: Session | null = null;
  private database: DatabaseService;
  private statusBarItem: vscode.StatusBarItem;
  private updateTimer: NodeJS.Timeout | null = null;
  private autoSaveTimer: NodeJS.Timeout | null = null;
  private lastBreakSuggestion: number = 0;
  private lastFlowNotification: number = 0;
  private flowAnalyzer: FlowStateAnalyzer;

  constructor(private context: vscode.ExtensionContext) {
    this.database = new DatabaseService(context.globalStorageUri.fsPath);
    this.flowAnalyzer = new FlowStateAnalyzer();
    
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
    this.flowAnalyzer.reset(); // Reset flow analysis for new session
    
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

  async endSession(): Promise<SessionData | null> {
    if (!this.currentSession) {
      vscode.window.showWarningMessage('No session to end');
      return null;
    }

    this.currentSession.end();
    const sessionData = this.currentSession.toJSON();
    
    await this.database.saveSession(sessionData);
    this.stopTimers();
    
    // Show enhanced notification
    await NotificationService.showSessionEndNotification(sessionData);
    
    const result = sessionData;
    this.currentSession = null;
    this.updateStatusBar();
    
    console.log(`Session ended: ${result.id}, Duration: ${result.duration}ms`);
    return result;
  }

  recordActivity(file?: string): void {
    if (this.currentSession) {
      this.currentSession.recordActivity(file);
      this.flowAnalyzer.recordKeystroke(file);
      this.checkAutoResume();
    }
  }

  getCurrentSession(): Session | null {
    return this.currentSession;
  }

  async getSessionHistory(days: number = 30): Promise<SessionData[]> {
    return this.database.getRecentSessions(days);
  }

  getFlowInsights(): string[] {
    return this.flowAnalyzer.getFlowStateInsights();
  }

  getFlowMetrics() {
    return this.flowAnalyzer.analyzeCurrentFlowState();
  }

  private startTimers(): void {
    // Update status bar every 5 seconds
    this.updateTimer = setInterval(() => {
      this.updateStatusBar();
      this.checkIdleTimeout();
      this.checkBreakSuggestion();
      this.checkFlowState();
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

  private checkBreakSuggestion(): void {
    if (!this.currentSession?.isActive || this.currentSession.isPaused) {
      return;
    }

    const sessionData = this.currentSession.toJSON();
    const now = Date.now();
    
    // Suggest breaks every 45 minutes, but not more than once every 30 minutes
    if (sessionData.duration > 45 * 60 * 1000 && 
        now - this.lastBreakSuggestion > 30 * 60 * 1000) {
      this.lastBreakSuggestion = now;
      NotificationService.showBreakSuggestion(sessionData.duration);
    }
  }

  private checkFlowState(): void {
    if (!this.currentSession?.isActive || this.currentSession.isPaused) {
      this.flowAnalyzer.recordInactivity();
      return;
    }

    const config = vscode.workspace.getConfiguration('mindfulCode');
    const enableFlowDetection = config.get<boolean>('enableFlowDetection', true);
    
    if (!enableFlowDetection) {
      return;
    }

    // Use advanced flow state analysis
    const flowMetrics = this.flowAnalyzer.analyzeCurrentFlowState();
    const isInFlow = flowMetrics.flowProbability > 0.7;
    const now = Date.now();
    
    if (isInFlow && !this.currentSession.flowStateDetected) {
      // Entering flow state
      this.currentSession.flowStateDetected = true;
      
      if (now - this.lastFlowNotification > 20 * 60 * 1000) { // Don't spam notifications
        this.lastFlowNotification = now;
        NotificationService.showFlowStateNotification(flowMetrics.flowDuration);
      }
    } else if (!isInFlow && this.currentSession.flowStateDetected) {
      // Exiting flow state - could show insights
      this.currentSession.flowStateDetected = false;
    }

    // Update flow duration with accurate measurement
    if (isInFlow) {
      this.currentSession.flowStateDuration = flowMetrics.flowDuration;
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
      const flowIndicator = session.flowStateDetected ? '🌊' : '';
      this.statusBarItem.text = `$(pulse) ${duration}m ${flowIndicator}`;
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
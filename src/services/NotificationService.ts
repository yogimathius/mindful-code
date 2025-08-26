import * as vscode from 'vscode';
import { SessionData } from '../models/Session';

export interface SessionSummary {
  duration: string;
  productivity: string;
  focusScore: number;
  keystrokes: number;
  filesWorked: number;
  flowStateTime: string;
  recommendations: string[];
}

export class NotificationService {
  private static formatDuration(milliseconds: number): string {
    const minutes = Math.floor(milliseconds / 60000);
    const hours = Math.floor(minutes / 60);
    const remainingMinutes = minutes % 60;
    
    if (hours > 0) {
      return `${hours}h ${remainingMinutes}m`;
    }
    return `${minutes}m`;
  }

  private static calculateProductivityScore(session: SessionData): number {
    if (session.duration === 0) return 0;
    
    const activeRatio = session.activeTime / session.duration;
    const keystrokesPerMinute = session.keystrokes / (session.duration / 60000);
    const flowRatio = session.flowStateDuration / session.duration;
    
    // Weighted score: 40% active time, 30% keystrokes rate, 30% flow state
    const score = (activeRatio * 0.4) + 
                  (Math.min(keystrokesPerMinute / 100, 1) * 0.3) + 
                  (flowRatio * 0.3);
    
    return Math.round(score * 100);
  }

  private static getProductivityLabel(score: number): string {
    if (score >= 80) return 'Excellent';
    if (score >= 65) return 'Good';
    if (score >= 50) return 'Average';
    if (score >= 30) return 'Below Average';
    return 'Poor';
  }

  private static generateRecommendations(session: SessionData, focusScore: number): string[] {
    const recommendations: string[] = [];
    
    const sessionMinutes = session.duration / 60000;
    const activeRatio = session.activeTime / session.duration;
    const flowRatio = session.flowStateDuration / session.duration;
    
    // Session length recommendations
    if (sessionMinutes < 25) {
      recommendations.push('Try longer focused sessions (25-45 minutes) for deeper work');
    } else if (sessionMinutes > 90) {
      recommendations.push('Consider shorter sessions with breaks to maintain focus');
    }
    
    // Activity recommendations
    if (activeRatio < 0.6) {
      recommendations.push('Minimize distractions to increase active coding time');
    }
    
    // Flow state recommendations
    if (flowRatio < 0.3) {
      recommendations.push('Create a distraction-free environment to achieve flow state');
    } else if (flowRatio > 0.7) {
      recommendations.push('Great flow state! Try to replicate these conditions');
    }
    
    // Focus score recommendations
    if (focusScore < 50) {
      recommendations.push('Consider using focus techniques like Pomodoro timer');
    }
    
    // File switching recommendations
    if (session.filesWorkedOn.length > 10) {
      recommendations.push('Try focusing on fewer files per session for deeper work');
    }
    
    return recommendations.slice(0, 3); // Limit to 3 recommendations
  }

  static createSessionSummary(session: SessionData): SessionSummary {
    const focusScore = this.calculateProductivityScore(session);
    
    return {
      duration: this.formatDuration(session.duration),
      productivity: this.getProductivityLabel(focusScore),
      focusScore,
      keystrokes: session.keystrokes,
      filesWorked: session.filesWorkedOn.length,
      flowStateTime: this.formatDuration(session.flowStateDuration),
      recommendations: this.generateRecommendations(session, focusScore)
    };
  }

  static async showSessionEndNotification(session: SessionData): Promise<void> {
    const config = vscode.workspace.getConfiguration('mindfulCode');
    const showNotifications = config.get<boolean>('showNotifications', true);
    
    if (!showNotifications) {
      return;
    }

    const summary = this.createSessionSummary(session);
    
    // Create detailed message
    const message = [
      `📊 Session Complete: ${summary.duration}`,
      `🎯 Focus Score: ${summary.focusScore}% (${summary.productivity})`,
      `⌨️  ${summary.keystrokes} keystrokes across ${summary.filesWorked} files`
    ].join('\n');

    const action = await vscode.window.showInformationMessage(
      message,
      'View Details',
      'View Recommendations'
    );

    if (action === 'View Details') {
      this.showDetailedSummary(summary);
    } else if (action === 'View Recommendations') {
      this.showRecommendations(summary);
    }
  }

  static async showDetailedSummary(summary: SessionSummary): Promise<void> {
    const details = [
      `🕐 Duration: ${summary.duration}`,
      `🎯 Focus Score: ${summary.focusScore}% (${summary.productivity})`,
      `⌨️  Keystrokes: ${summary.keystrokes}`,
      `📁 Files Worked: ${summary.filesWorked}`,
      `🌊 Flow State: ${summary.flowStateTime}`,
      '',
      '💡 Recommendations:',
      ...summary.recommendations.map(rec => `• ${rec}`)
    ].join('\n');

    await vscode.window.showInformationMessage(details, { modal: true });
  }

  static async showRecommendations(summary: SessionSummary): Promise<void> {
    if (summary.recommendations.length === 0) {
      await vscode.window.showInformationMessage('Great session! Keep up the good work! 🎉');
      return;
    }

    const message = [
      '💡 Recommendations for your next session:',
      '',
      ...summary.recommendations.map((rec, index) => `${index + 1}. ${rec}`)
    ].join('\n');

    await vscode.window.showInformationMessage(message, { modal: true });
  }

  static async showFlowStateNotification(duration: number): Promise<void> {
    const config = vscode.workspace.getConfiguration('mindfulCode');
    const showNotifications = config.get<boolean>('showNotifications', true);
    
    if (!showNotifications) {
      return;
    }

    const durationText = this.formatDuration(duration);
    await vscode.window.showInformationMessage(
      `🌊 Flow State Detected! You've been in flow for ${durationText}. Keep going! 🚀`
    );
  }

  static async showBreakSuggestion(sessionDuration: number): Promise<void> {
    const config = vscode.workspace.getConfiguration('mindfulCode');
    const showNotifications = config.get<boolean>('showNotifications', true);
    
    if (!showNotifications) {
      return;
    }

    const hours = Math.floor(sessionDuration / 3600000);
    
    let message = '☕ Consider taking a short break to maintain focus and prevent burnout.';
    
    if (hours >= 2) {
      message = '🚶‍♂️ You\'ve been coding for over 2 hours. Take a longer break and stretch!';
    } else if (hours >= 1) {
      message = '☕ You\'ve been focused for over an hour. A short break would help refresh your mind.';
    }

    const action = await vscode.window.showWarningMessage(
      message,
      'Take Break (5 min)',
      'Ignore'
    );

    if (action === 'Take Break (5 min)') {
      // TODO: Implement break timer
      vscode.window.showInformationMessage('⏱️ Break timer started! Come back in 5 minutes.');
    }
  }

  static async showWeeklySummary(stats: {
    totalSessions: number;
    totalCodingTime: number;
    averageSessionLength: number;
    totalFlowTime: number;
    productivityTrend: 'up' | 'down' | 'stable';
  }): Promise<void> {
    const totalHours = Math.round(stats.totalCodingTime / 3600000 * 10) / 10;
    const avgMinutes = Math.round(stats.averageSessionLength / 60000);
    const flowHours = Math.round(stats.totalFlowTime / 3600000 * 10) / 10;
    
    const trendEmoji = stats.productivityTrend === 'up' ? '📈' : 
                     stats.productivityTrend === 'down' ? '📉' : '📊';
    
    const message = [
      '📈 Weekly Summary:',
      `• ${stats.totalSessions} coding sessions`,
      `• ${totalHours}h total coding time`,
      `• ${avgMinutes}m average session length`,
      `• ${flowHours}h in flow state`,
      `${trendEmoji} Productivity trending ${stats.productivityTrend}`
    ].join('\n');

    await vscode.window.showInformationMessage(message, { modal: true });
  }
}
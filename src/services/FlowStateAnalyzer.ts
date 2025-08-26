import { SessionData } from '../models/Session';

export interface FlowStateMetrics {
  typingRhythm: number;
  focusConsistency: number;
  contextSwitching: number;
  errorRate: number;
  flowProbability: number;
  flowDuration: number;
}

export interface TypingPattern {
  timestamp: number;
  keystroke: boolean;
  file?: string;
}

export class FlowStateAnalyzer {
  private typingPatterns: TypingPattern[] = [];
  private fileChanges: Array<{ timestamp: number; file: string }> = [];
  private readonly maxPatternHistory = 1000; // Keep last 1000 keystrokes
  private readonly flowThreshold = 0.7; // 70% confidence for flow state

  recordKeystroke(file?: string): void {
    const now = Date.now();
    
    // Add keystroke pattern
    this.typingPatterns.push({
      timestamp: now,
      keystroke: true,
      file
    });

    // Track file changes
    if (file && (this.fileChanges.length === 0 || 
                this.fileChanges[this.fileChanges.length - 1].file !== file)) {
      this.fileChanges.push({ timestamp: now, file });
    }

    // Trim old patterns to keep memory usage reasonable
    if (this.typingPatterns.length > this.maxPatternHistory) {
      this.typingPatterns = this.typingPatterns.slice(-this.maxPatternHistory);
    }

    // Trim old file changes (keep last 50)
    if (this.fileChanges.length > 50) {
      this.fileChanges = this.fileChanges.slice(-50);
    }
  }

  recordInactivity(): void {
    const now = Date.now();
    this.typingPatterns.push({
      timestamp: now,
      keystroke: false
    });
  }

  analyzeCurrentFlowState(timeWindowMs: number = 10 * 60 * 1000): FlowStateMetrics {
    const now = Date.now();
    const windowStart = now - timeWindowMs;
    
    // Filter patterns to current time window
    const recentPatterns = this.typingPatterns.filter(p => p.timestamp >= windowStart);
    const recentFileChanges = this.fileChanges.filter(f => f.timestamp >= windowStart);

    if (recentPatterns.length < 10) {
      return this.getDefaultMetrics();
    }

    const typingRhythm = this.calculateTypingRhythm(recentPatterns);
    const focusConsistency = this.calculateFocusConsistency(recentPatterns);
    const contextSwitching = this.calculateContextSwitching(recentFileChanges, timeWindowMs);
    const errorRate = this.estimateErrorRate(recentPatterns);
    
    const flowProbability = this.calculateFlowProbability({
      typingRhythm,
      focusConsistency,
      contextSwitching,
      errorRate
    });

    const flowDuration = flowProbability > this.flowThreshold ? 
                        this.estimateFlowDuration(recentPatterns) : 0;

    return {
      typingRhythm,
      focusConsistency,
      contextSwitching,
      errorRate,
      flowProbability,
      flowDuration
    };
  }

  private calculateTypingRhythm(patterns: TypingPattern[]): number {
    const keystrokes = patterns.filter(p => p.keystroke);
    
    if (keystrokes.length < 5) {
      return 0;
    }

    // Calculate intervals between keystrokes
    const intervals: number[] = [];
    for (let i = 1; i < keystrokes.length; i++) {
      intervals.push(keystrokes[i].timestamp - keystrokes[i - 1].timestamp);
    }

    // Calculate rhythm consistency (lower variance = better rhythm)
    const meanInterval = intervals.reduce((a, b) => a + b, 0) / intervals.length;
    const variance = intervals.reduce((sum, interval) => {
      return sum + Math.pow(interval - meanInterval, 2);
    }, 0) / intervals.length;

    const stdDev = Math.sqrt(variance);
    const coefficientOfVariation = stdDev / meanInterval;

    // Convert to 0-1 score (lower variation = higher score)
    return Math.max(0, 1 - coefficientOfVariation);
  }

  private calculateFocusConsistency(patterns: TypingPattern[]): number {
    if (patterns.length < 10) {
      return 0;
    }

    // Analyze activity distribution over time
    const timeSlots = 20; // Divide time window into 20 slots
    const windowDuration = patterns[patterns.length - 1].timestamp - patterns[0].timestamp;
    const slotDuration = windowDuration / timeSlots;
    
    const activityPerSlot = new Array(timeSlots).fill(0);
    
    patterns.forEach(pattern => {
      if (pattern.keystroke) {
        const slotIndex = Math.floor(
          (pattern.timestamp - patterns[0].timestamp) / slotDuration
        );
        if (slotIndex < timeSlots) {
          activityPerSlot[slotIndex]++;
        }
      }
    });

    // Calculate consistency (even distribution = higher consistency)
    const totalActivity = activityPerSlot.reduce((a, b) => a + b, 0);
    const expectedActivityPerSlot = totalActivity / timeSlots;
    
    const variance = activityPerSlot.reduce((sum, activity) => {
      return sum + Math.pow(activity - expectedActivityPerSlot, 2);
    }, 0) / timeSlots;

    // Normalize to 0-1 score
    const maxPossibleVariance = Math.pow(totalActivity, 2);
    return Math.max(0, 1 - (variance / maxPossibleVariance));
  }

  private calculateContextSwitching(fileChanges: Array<{ timestamp: number; file: string }>, windowMs: number): number {
    if (fileChanges.length <= 1) {
      return 1; // No switching = perfect score
    }

    // Count file switches per minute
    const windowMinutes = windowMs / (60 * 1000);
    const switchesPerMinute = (fileChanges.length - 1) / windowMinutes;

    // Ideal range: 0-2 switches per minute
    // More switches = lower score (more context switching)
    if (switchesPerMinute <= 2) {
      return 1 - (switchesPerMinute / 4); // Scale 0-2 to 1-0.5
    } else {
      return Math.max(0, 0.5 - (switchesPerMinute - 2) / 10); // Rapid decline after 2
    }
  }

  private estimateErrorRate(patterns: TypingPattern[]): number {
    // Simple heuristic: rapid delete sequences indicate errors/corrections
    const keystrokes = patterns.filter(p => p.keystroke);
    
    if (keystrokes.length < 20) {
      return 0.5; // Default assumption
    }

    // Look for rapid keystroke bursts (potential corrections)
    let rapidBursts = 0;
    let consecutiveQuick = 0;
    
    for (let i = 1; i < keystrokes.length; i++) {
      const interval = keystrokes[i].timestamp - keystrokes[i - 1].timestamp;
      
      if (interval < 100) { // Very rapid typing (< 100ms between keys)
        consecutiveQuick++;
      } else {
        if (consecutiveQuick >= 3) {
          rapidBursts++;
        }
        consecutiveQuick = 0;
      }
    }

    // More bursts = higher error rate
    const errorRate = Math.min(1, rapidBursts / (keystrokes.length / 50));
    return 1 - errorRate; // Return as quality score (1 = low errors)
  }

  private calculateFlowProbability(metrics: {
    typingRhythm: number;
    focusConsistency: number;
    contextSwitching: number;
    errorRate: number;
  }): number {
    // Weighted combination of all metrics
    const weights = {
      typingRhythm: 0.25,
      focusConsistency: 0.35,
      contextSwitching: 0.25,
      errorRate: 0.15
    };

    return (
      metrics.typingRhythm * weights.typingRhythm +
      metrics.focusConsistency * weights.focusConsistency +
      metrics.contextSwitching * weights.contextSwitching +
      metrics.errorRate * weights.errorRate
    );
  }

  private estimateFlowDuration(patterns: TypingPattern[]): number {
    if (patterns.length < 2) {
      return 0;
    }

    // Find the longest continuous period of consistent activity
    const keystrokes = patterns.filter(p => p.keystroke);
    
    if (keystrokes.length < 10) {
      return 0;
    }

    let maxFlowDuration = 0;
    let currentFlowStart = keystrokes[0].timestamp;
    let lastActivityTime = keystrokes[0].timestamp;

    for (let i = 1; i < keystrokes.length; i++) {
      const gap = keystrokes[i].timestamp - lastActivityTime;
      
      if (gap > 30000) { // Gap > 30 seconds breaks flow
        const flowDuration = lastActivityTime - currentFlowStart;
        maxFlowDuration = Math.max(maxFlowDuration, flowDuration);
        currentFlowStart = keystrokes[i].timestamp;
      }
      
      lastActivityTime = keystrokes[i].timestamp;
    }

    // Check final flow period
    const finalFlowDuration = lastActivityTime - currentFlowStart;
    maxFlowDuration = Math.max(maxFlowDuration, finalFlowDuration);

    return maxFlowDuration;
  }

  private getDefaultMetrics(): FlowStateMetrics {
    return {
      typingRhythm: 0,
      focusConsistency: 0,
      contextSwitching: 1,
      errorRate: 0.5,
      flowProbability: 0,
      flowDuration: 0
    };
  }

  isInFlowState(timeWindowMs: number = 10 * 60 * 1000): boolean {
    const metrics = this.analyzeCurrentFlowState(timeWindowMs);
    return metrics.flowProbability > this.flowThreshold;
  }

  getFlowStateInsights(): string[] {
    const metrics = this.analyzeCurrentFlowState();
    const insights: string[] = [];

    if (metrics.typingRhythm < 0.5) {
      insights.push('Try to maintain a steady typing rhythm for better flow');
    }

    if (metrics.focusConsistency < 0.6) {
      insights.push('Minimize interruptions to maintain consistent focus');
    }

    if (metrics.contextSwitching < 0.7) {
      insights.push('Reduce file switching to stay in the flow zone');
    }

    if (metrics.errorRate < 0.7) {
      insights.push('Slow down slightly to reduce errors and maintain flow');
    }

    if (insights.length === 0) {
      insights.push('Great flow state conditions! Keep up the momentum');
    }

    return insights;
  }

  reset(): void {
    this.typingPatterns = [];
    this.fileChanges = [];
  }
}
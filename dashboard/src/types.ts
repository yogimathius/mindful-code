export interface SessionData {
  id: string;
  startTime: Date;
  endTime?: Date;
  duration: number;
  isActive: boolean;
  isPaused: boolean;
  pausedDuration: number;
  filesWorkedOn: string[];
  keystrokes: number;
  activeTime: number;
  flowStateDetected: boolean;
  flowStateDuration: number;
  interruptions: number;
}

export interface FlowMetrics {
  typingRhythm: number;
  focusIntensity: number;
  errorRate: number;
  contextSwitching: number;
  confidenceScore: number;
}

export interface DashboardStats {
  totalSessions: number;
  totalCodingTime: number;
  totalFlowTime: number;
  averageSessionLength: number;
  productivityScore: number;
  peakHours: number[];
  mostProductiveDays: string[];
  weeklyTrend: 'up' | 'down' | 'stable';
  dailyStats: Array<{
    date: string;
    sessions: number;
    codingTime: number;
    flowTime: number;
  }>;
}

export interface TimeRange {
  start: Date;
  end: Date;
  label: string;
}
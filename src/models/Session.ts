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

export interface SessionSummary {
  totalSessions: number;
  totalCodingTime: number;
  totalFlowTime: number;
  averageSessionLength: number;
  productivityScore: number;
  peakHours: number[];
  mostProductiveDays: string[];
}

export class Session implements SessionData {
  id: string;
  startTime: Date;
  endTime?: Date;
  duration: number = 0;
  isActive: boolean = false;
  isPaused: boolean = false;
  pausedDuration: number = 0;
  filesWorkedOn: string[] = [];
  keystrokes: number = 0;
  activeTime: number = 0;
  flowStateDetected: boolean = false;
  flowStateDuration: number = 0;
  interruptions: number = 0;

  private pauseStartTime?: Date;
  private lastActivityTime: Date;

  constructor(id?: string) {
    this.id = id || this.generateId();
    this.startTime = new Date();
    this.lastActivityTime = new Date();
  }

  private generateId(): string {
    return `session_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }

  start(): void {
    this.isActive = true;
    this.isPaused = false;
    this.startTime = new Date();
    this.lastActivityTime = new Date();
  }

  pause(): void {
    if (this.isActive && !this.isPaused) {
      this.isPaused = true;
      this.pauseStartTime = new Date();
      this.updateDuration();
    }
  }

  resume(): void {
    if (this.isPaused && this.pauseStartTime) {
      const pauseDuration = Date.now() - this.pauseStartTime.getTime();
      this.pausedDuration += pauseDuration;
      this.isPaused = false;
      this.pauseStartTime = undefined;
      this.lastActivityTime = new Date();
    }
  }

  end(): void {
    if (this.isPaused) {
      this.resume();
    }
    this.isActive = false;
    this.endTime = new Date();
    this.updateDuration();
  }

  recordActivity(file?: string): void {
    if (!this.isActive || this.isPaused) {
      return;
    }

    this.lastActivityTime = new Date();
    this.keystrokes++;

    if (file && !this.filesWorkedOn.includes(file)) {
      this.filesWorkedOn.push(file);
    }
  }

  private updateDuration(): void {
    if (this.endTime) {
      this.duration = this.endTime.getTime() - this.startTime.getTime() - this.pausedDuration;
    } else {
      this.duration = Date.now() - this.startTime.getTime() - this.pausedDuration;
    }
    this.activeTime = this.duration;
  }

  shouldAutoPause(idleTimeoutMs: number): boolean {
    if (!this.isActive || this.isPaused) {
      return false;
    }
    
    const idleTime = Date.now() - this.lastActivityTime.getTime();
    return idleTime > idleTimeoutMs;
  }

  toJSON(): SessionData {
    this.updateDuration();
    return {
      id: this.id,
      startTime: this.startTime,
      endTime: this.endTime,
      duration: this.duration,
      isActive: this.isActive,
      isPaused: this.isPaused,
      pausedDuration: this.pausedDuration,
      filesWorkedOn: [...this.filesWorkedOn],
      keystrokes: this.keystrokes,
      activeTime: this.activeTime,
      flowStateDetected: this.flowStateDetected,
      flowStateDuration: this.flowStateDuration,
      interruptions: this.interruptions,
    };
  }
}
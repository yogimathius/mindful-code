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
export declare class Session implements SessionData {
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
    private pauseStartTime?;
    private lastActivityTime;
    constructor(id?: string);
    private generateId;
    start(): void;
    pause(): void;
    resume(): void;
    end(): void;
    recordActivity(file?: string): void;
    private updateDuration;
    shouldAutoPause(idleTimeoutMs: number): boolean;
    toJSON(): SessionData;
}
//# sourceMappingURL=Session.d.ts.map
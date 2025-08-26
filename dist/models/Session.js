"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.Session = void 0;
class Session {
    constructor(id) {
        this.duration = 0;
        this.isActive = false;
        this.isPaused = false;
        this.pausedDuration = 0;
        this.filesWorkedOn = [];
        this.keystrokes = 0;
        this.activeTime = 0;
        this.flowStateDetected = false;
        this.flowStateDuration = 0;
        this.interruptions = 0;
        this.id = id || this.generateId();
        this.startTime = new Date();
        this.lastActivityTime = new Date();
    }
    generateId() {
        return `session_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
    }
    start() {
        this.isActive = true;
        this.isPaused = false;
        this.startTime = new Date();
        this.lastActivityTime = new Date();
    }
    pause() {
        if (this.isActive && !this.isPaused) {
            this.isPaused = true;
            this.pauseStartTime = new Date();
            this.updateDuration();
        }
    }
    resume() {
        if (this.isPaused && this.pauseStartTime) {
            const pauseDuration = Date.now() - this.pauseStartTime.getTime();
            this.pausedDuration += pauseDuration;
            this.isPaused = false;
            this.pauseStartTime = undefined;
            this.lastActivityTime = new Date();
        }
    }
    end() {
        if (this.isPaused) {
            this.resume();
        }
        this.isActive = false;
        this.endTime = new Date();
        this.updateDuration();
    }
    recordActivity(file) {
        if (!this.isActive || this.isPaused) {
            return;
        }
        this.lastActivityTime = new Date();
        this.keystrokes++;
        if (file && !this.filesWorkedOn.includes(file)) {
            this.filesWorkedOn.push(file);
        }
    }
    updateDuration() {
        if (this.endTime) {
            this.duration = this.endTime.getTime() - this.startTime.getTime() - this.pausedDuration;
        }
        else {
            this.duration = Date.now() - this.startTime.getTime() - this.pausedDuration;
        }
        this.activeTime = this.duration;
    }
    shouldAutoPause(idleTimeoutMs) {
        if (!this.isActive || this.isPaused) {
            return false;
        }
        const idleTime = Date.now() - this.lastActivityTime.getTime();
        return idleTime > idleTimeoutMs;
    }
    toJSON() {
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
exports.Session = Session;
//# sourceMappingURL=Session.js.map
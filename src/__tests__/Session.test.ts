import { Session, SessionData } from '../models/Session';

describe('Session', () => {
  let session: Session;

  beforeEach(() => {
    session = new Session();
    jest.clearAllMocks();
  });

  describe('constructor', () => {
    it('should create a session with generated ID if none provided', () => {
      const session = new Session();
      expect(session.id).toBeDefined();
      expect(session.id).toMatch(/^session_\d+_[a-z0-9]+$/);
    });

    it('should create a session with provided ID', () => {
      const customId = 'test-session-123';
      const session = new Session(customId);
      expect(session.id).toBe(customId);
    });

    it('should initialize with default values', () => {
      expect(session.isActive).toBe(false);
      expect(session.isPaused).toBe(false);
      expect(session.duration).toBe(0);
      expect(session.keystrokes).toBe(0);
      expect(session.filesWorkedOn).toEqual([]);
    });
  });

  describe('start', () => {
    it('should activate the session', () => {
      session.start();
      expect(session.isActive).toBe(true);
      expect(session.isPaused).toBe(false);
      expect(session.startTime).toBeInstanceOf(Date);
    });
  });

  describe('pause and resume', () => {
    beforeEach(() => {
      session.start();
    });

    it('should pause an active session', () => {
      session.pause();
      expect(session.isPaused).toBe(true);
    });

    it('should not pause an inactive session', () => {
      session.isActive = false;
      session.pause();
      expect(session.isPaused).toBe(false);
    });

    it('should resume a paused session', () => {
      session.pause();
      expect(session.isPaused).toBe(true);
      
      session.resume();
      expect(session.isPaused).toBe(false);
    });

    it('should track paused duration', async () => {
      session.pause();
      
      // Wait a small amount of time
      await new Promise(resolve => setTimeout(resolve, 10));
      
      session.resume();
      expect(session.pausedDuration).toBeGreaterThan(0);
    });
  });

  describe('end', () => {
    beforeEach(() => {
      session.start();
    });

    it('should end an active session', () => {
      session.end();
      expect(session.isActive).toBe(false);
      expect(session.endTime).toBeInstanceOf(Date);
    });

    it('should resume before ending if paused', () => {
      session.pause();
      const pausedDurationBefore = session.pausedDuration;
      
      session.end();
      expect(session.isActive).toBe(false);
      expect(session.isPaused).toBe(false);
    });
  });

  describe('recordActivity', () => {
    beforeEach(() => {
      session.start();
    });

    it('should increment keystrokes when active', () => {
      const initialKeystrokes = session.keystrokes;
      session.recordActivity();
      expect(session.keystrokes).toBe(initialKeystrokes + 1);
    });

    it('should not increment keystrokes when paused', () => {
      session.pause();
      const initialKeystrokes = session.keystrokes;
      session.recordActivity();
      expect(session.keystrokes).toBe(initialKeystrokes);
    });

    it('should not increment keystrokes when inactive', () => {
      session.end();
      const initialKeystrokes = session.keystrokes;
      session.recordActivity();
      expect(session.keystrokes).toBe(initialKeystrokes);
    });

    it('should track files worked on', () => {
      const file1 = '/path/to/file1.ts';
      const file2 = '/path/to/file2.ts';
      
      session.recordActivity(file1);
      session.recordActivity(file2);
      session.recordActivity(file1); // Duplicate should not be added
      
      expect(session.filesWorkedOn).toEqual([file1, file2]);
    });
  });

  describe('shouldAutoPause', () => {
    beforeEach(() => {
      session.start();
    });

    it('should return true when idle time exceeds timeout', async () => {
      const idleTimeout = 100; // 100ms
      
      // Wait for idle timeout
      await new Promise(resolve => setTimeout(resolve, idleTimeout + 10));
      
      expect(session.shouldAutoPause(idleTimeout)).toBe(true);
    });

    it('should return false when recently active', () => {
      const idleTimeout = 1000; // 1 second
      session.recordActivity();
      
      expect(session.shouldAutoPause(idleTimeout)).toBe(false);
    });

    it('should return false when session is paused', () => {
      session.pause();
      const idleTimeout = 0;
      
      expect(session.shouldAutoPause(idleTimeout)).toBe(false);
    });

    it('should return false when session is inactive', () => {
      session.end();
      const idleTimeout = 0;
      
      expect(session.shouldAutoPause(idleTimeout)).toBe(false);
    });
  });

  describe('toJSON', () => {
    it('should return serializable session data', () => {
      session.start();
      session.recordActivity('/test/file.ts');
      session.end();
      
      const json = session.toJSON();
      
      expect(json).toEqual({
        id: session.id,
        startTime: session.startTime,
        endTime: session.endTime,
        duration: expect.any(Number),
        isActive: false,
        isPaused: false,
        pausedDuration: 0,
        filesWorkedOn: ['/test/file.ts'],
        keystrokes: 1,
        activeTime: expect.any(Number),
        flowStateDetected: false,
        flowStateDuration: 0,
        interruptions: 0,
      });
    });

    it('should update duration before serializing', async () => {
      session.start();
      // Wait a small amount to ensure time has passed
      await new Promise(resolve => setTimeout(resolve, 10));
      const json = session.toJSON();
      expect(json.duration).toBeGreaterThan(0);
    });
  });

  describe('duration calculation', () => {
    it('should calculate duration excluding paused time', async () => {
      session.start();
      
      // Active for a bit
      await new Promise(resolve => setTimeout(resolve, 50));
      
      session.pause();
      
      // Paused for a bit
      await new Promise(resolve => setTimeout(resolve, 50));
      
      session.resume();
      session.end();
      
      const json = session.toJSON();
      
      // Duration should be less than total elapsed time due to pause
      const totalElapsed = json.endTime!.getTime() - json.startTime.getTime();
      expect(json.duration).toBeLessThan(totalElapsed);
      expect(json.duration).toBeGreaterThan(0);
      expect(json.pausedDuration).toBeGreaterThan(0);
    });
  });
});
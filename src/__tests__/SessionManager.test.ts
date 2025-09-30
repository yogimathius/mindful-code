import { SessionManager } from '../services/SessionManager';
import { DatabaseService } from '../services/DatabaseService';

// Mock DatabaseService
jest.mock('../services/DatabaseService');

const vscode = require('vscode');

describe('SessionManager', () => {
  let sessionManager: SessionManager;
  let mockContext: any;
  let mockDatabaseService: jest.Mocked<DatabaseService>;

  beforeEach(() => {
    mockContext = {
      globalStorageUri: { fsPath: '/tmp/test-storage' },
      subscriptions: [],
    } as any;

    mockDatabaseService = new DatabaseService('') as jest.Mocked<DatabaseService>;
    mockDatabaseService.saveSession = jest.fn().mockResolvedValue(undefined);
    mockDatabaseService.getRecentSessions = jest.fn().mockResolvedValue([]);

    sessionManager = new SessionManager(mockContext);
    (sessionManager as any).database = mockDatabaseService;

    jest.clearAllMocks();
  });

  afterEach(() => {
    sessionManager.dispose();
  });

  describe('startSession', () => {
    it('should start a new session when none exists', async () => {
      await sessionManager.startSession();
      
      const currentSession = sessionManager.getCurrentSession();
      expect(currentSession).not.toBeNull();
      expect(currentSession?.isActive).toBe(true);
      expect(currentSession?.isPaused).toBe(false);
      expect(mockDatabaseService.saveSession).toHaveBeenCalledTimes(1);
    });

    it('should show warning when session already active', async () => {
      await sessionManager.startSession();
      await sessionManager.startSession();
      
      expect(vscode.window.showWarningMessage).toHaveBeenCalledWith('Session already active');
    });

    it('should resume paused session instead of creating new one', async () => {
      await sessionManager.startSession();
      sessionManager.pauseSession();
      
      const pausedSession = sessionManager.getCurrentSession();
      const sessionId = pausedSession?.id;
      
      await sessionManager.startSession();
      
      const resumedSession = sessionManager.getCurrentSession();
      expect(resumedSession?.id).toBe(sessionId);
      expect(resumedSession?.isActive).toBe(true);
      expect(resumedSession?.isPaused).toBe(false);
    });
  });

  describe('pauseSession', () => {
    it('should pause an active session', async () => {
      await sessionManager.startSession();
      sessionManager.pauseSession();
      
      const currentSession = sessionManager.getCurrentSession();
      expect(currentSession?.isPaused).toBe(true);
      expect(currentSession?.isActive).toBe(true);
    });

    it('should show warning when no active session to pause', () => {
      sessionManager.pauseSession();
      expect(vscode.window.showWarningMessage).toHaveBeenCalledWith('No active session to pause');
    });
  });

  describe('resumeSession', () => {
    it('should resume a paused session', async () => {
      await sessionManager.startSession();
      sessionManager.pauseSession();
      sessionManager.resumeSession();
      
      const currentSession = sessionManager.getCurrentSession();
      expect(currentSession?.isPaused).toBe(false);
      expect(currentSession?.isActive).toBe(true);
    });

    it('should show warning when no paused session to resume', () => {
      sessionManager.resumeSession();
      expect(vscode.window.showWarningMessage).toHaveBeenCalledWith('No paused session to resume');
    });
  });

  describe('endSession', () => {
    it('should end an active session and return session data', async () => {
      await sessionManager.startSession();
      const sessionData = await sessionManager.endSession();
      
      expect(sessionData).not.toBeNull();
      expect(sessionData?.isActive).toBe(false);
      expect(sessionData?.endTime).toBeInstanceOf(Date);
      expect(sessionManager.getCurrentSession()).toBeNull();
      expect(mockDatabaseService.saveSession).toHaveBeenCalledWith(sessionData);
    });

    it('should show warning when no session to end', async () => {
      const result = await sessionManager.endSession();
      expect(result).toBeNull();
      expect(vscode.window.showWarningMessage).toHaveBeenCalledWith('No session to end');
    });

    it('should end a paused session', async () => {
      await sessionManager.startSession();
      sessionManager.pauseSession();
      
      const sessionData = await sessionManager.endSession();
      
      expect(sessionData).not.toBeNull();
      expect(sessionData?.isActive).toBe(false);
      expect(sessionData?.isPaused).toBe(false);
    });
  });

  describe('recordActivity', () => {
    it('should record activity when session is active', async () => {
      await sessionManager.startSession();
      const initialKeystrokes = sessionManager.getCurrentSession()?.keystrokes || 0;
      
      sessionManager.recordActivity('/test/file.ts');
      
      const currentSession = sessionManager.getCurrentSession();
      expect(currentSession?.keystrokes).toBe(initialKeystrokes + 1);
      expect(currentSession?.filesWorkedOn).toContain('/test/file.ts');
    });

    it('should not record activity when no session exists', () => {
      sessionManager.recordActivity('/test/file.ts');
      expect(sessionManager.getCurrentSession()).toBeNull();
    });

    it('should auto-resume paused session on activity', async () => {
      await sessionManager.startSession();
      sessionManager.pauseSession();
      
      sessionManager.recordActivity('/test/file.ts');
      
      const currentSession = sessionManager.getCurrentSession();
      expect(currentSession?.isPaused).toBe(false);
      expect(vscode.window.showInformationMessage).toHaveBeenCalledWith('Session auto-resumed');
    });
  });

  describe('getSessionHistory', () => {
    it('should return session history from database', async () => {
      const mockSessions = [
        { id: 'test1', startTime: new Date(), duration: 1000 },
        { id: 'test2', startTime: new Date(), duration: 2000 },
      ];
      mockDatabaseService.getRecentSessions.mockResolvedValue(mockSessions as any);
      
      const history = await sessionManager.getSessionHistory(7);
      
      expect(history).toEqual(mockSessions);
      expect(mockDatabaseService.getRecentSessions).toHaveBeenCalledWith(7);
    });
  });

  describe('idle timeout', () => {
    beforeEach(() => {
      jest.useFakeTimers();
    });

    afterEach(() => {
      jest.useRealTimers();
    });

    it('should auto-pause session after idle timeout', async () => {
      await sessionManager.startSession();
      
      // Fast-forward time to trigger idle timeout check
      jest.advanceTimersByTime(6000); // 6 seconds to trigger check
      
      // The session should still be active since no idle time has passed
      expect(sessionManager.getCurrentSession()?.isPaused).toBe(false);
      
      // Manually trigger idle timeout by advancing the session's last activity
      const session = sessionManager.getCurrentSession();
      if (session) {
        // Simulate 6 minutes of idle time (more than 5 minute timeout)
        (session as any).lastActivityTime = new Date(Date.now() - 6 * 60 * 1000);
      }
      
      // Trigger another timeout check
      jest.advanceTimersByTime(5000);
      
      expect(sessionManager.getCurrentSession()?.isPaused).toBe(true);
      expect(vscode.window.showInformationMessage).toHaveBeenCalledWith(
        'Session auto-paused after 5 minutes of inactivity'
      );
    });
  });
});
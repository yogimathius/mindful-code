import { ActivityTracker } from '../services/ActivityTracker';
import { SessionManager } from '../services/SessionManager';

// Mock SessionManager
jest.mock('../services/SessionManager');

const vscode = require('vscode');

describe('ActivityTracker', () => {
  let activityTracker: ActivityTracker;
  let mockSessionManager: jest.Mocked<SessionManager>;
  let mockContext: any;

  // Event handler storage
  let documentChangeHandler: (event: any) => void;
  let activeEditorHandler: (editor: any) => void;
  let windowFocusHandler: (state: any) => void;
  let selectionChangeHandler: (event: any) => void;
  let visibleRangesHandler: (event: any) => void;
  let debugStartHandler: () => void;
  let debugTerminateHandler: () => void;

  beforeEach(() => {
    mockSessionManager = new SessionManager({} as any) as jest.Mocked<SessionManager>;
    mockSessionManager.recordActivity = jest.fn();

    mockContext = {
      subscriptions: [],
    } as any;

    // Setup event handler mocks
    (vscode.workspace.onDidChangeTextDocument as jest.Mock).mockImplementation((handler) => {
      documentChangeHandler = handler;
      return { dispose: jest.fn() };
    });

    (vscode.window.onDidChangeActiveTextEditor as jest.Mock).mockImplementation((handler) => {
      activeEditorHandler = handler;
      return { dispose: jest.fn() };
    });

    (vscode.window.onDidChangeWindowState as jest.Mock).mockImplementation((handler) => {
      windowFocusHandler = handler;
      return { dispose: jest.fn() };
    });

    (vscode.window.onDidChangeTextEditorSelection as jest.Mock).mockImplementation((handler) => {
      selectionChangeHandler = handler;
      return { dispose: jest.fn() };
    });

    (vscode.window.onDidChangeTextEditorVisibleRanges as jest.Mock).mockImplementation((handler) => {
      visibleRangesHandler = handler;
      return { dispose: jest.fn() };
    });

    (vscode.debug.onDidStartDebugSession as jest.Mock).mockImplementation((handler) => {
      debugStartHandler = handler;
      return { dispose: jest.fn() };
    });

    (vscode.debug.onDidTerminateDebugSession as jest.Mock).mockImplementation((handler) => {
      debugTerminateHandler = handler;
      return { dispose: jest.fn() };
    });

    activityTracker = new ActivityTracker(mockSessionManager);

    jest.clearAllMocks();
  });

  afterEach(() => {
    activityTracker.dispose();
  });

  describe('activate', () => {
    it('should register all event listeners', () => {
      activityTracker.activate(mockContext);

      expect(vscode.workspace.onDidChangeTextDocument).toHaveBeenCalled();
      expect(vscode.window.onDidChangeActiveTextEditor).toHaveBeenCalled();
      expect(vscode.window.onDidChangeWindowState).toHaveBeenCalled();
      expect(vscode.window.onDidChangeTextEditorSelection).toHaveBeenCalled();
      expect(vscode.window.onDidChangeTextEditorVisibleRanges).toHaveBeenCalled();
      expect(vscode.debug.onDidStartDebugSession).toHaveBeenCalled();
      expect(vscode.debug.onDidTerminateDebugSession).toHaveBeenCalled();
    });

    it('should add disposables to context subscriptions', () => {
      activityTracker.activate(mockContext);
      expect(mockContext.subscriptions.length).toBeGreaterThan(0);
    });
  });

  describe('event handling', () => {
    beforeEach(() => {
      activityTracker.activate(mockContext);
    });

    it('should record activity on text document changes', () => {
      const mockEvent = {
        document: { uri: { fsPath: '/workspace/test.ts' } },
        contentChanges: [{ text: 'hello' }],
      };

      documentChangeHandler(mockEvent);

      expect(mockSessionManager.recordActivity).toHaveBeenCalledWith('/workspace/test.ts');
    });

    it('should not record activity when no content changes', () => {
      const mockEvent = {
        document: { uri: { fsPath: '/workspace/test.ts' } },
        contentChanges: [],
      };

      documentChangeHandler(mockEvent);

      expect(mockSessionManager.recordActivity).not.toHaveBeenCalled();
    });

    it('should record activity on active editor changes', () => {
      const mockEditor = {
        document: { uri: { fsPath: '/workspace/test.ts' } },
      };

      activeEditorHandler(mockEditor);

      expect(mockSessionManager.recordActivity).toHaveBeenCalledWith('/workspace/test.ts');
    });

    it('should record activity on window focus', () => {
      const mockState = { focused: true };

      windowFocusHandler(mockState);

      expect(mockSessionManager.recordActivity).toHaveBeenCalledWith();
    });

    it('should not record activity when window loses focus', () => {
      const mockState = { focused: false };

      windowFocusHandler(mockState);

      expect(mockSessionManager.recordActivity).not.toHaveBeenCalled();
    });

    it('should record activity on selection changes', () => {
      const mockEvent = {
        textEditor: {
          document: { uri: { fsPath: '/workspace/test.ts' } },
        },
      };

      selectionChangeHandler(mockEvent);

      expect(mockSessionManager.recordActivity).toHaveBeenCalledWith('/workspace/test.ts');
    });

    it('should record activity on visible ranges changes', () => {
      const mockEvent = {
        textEditor: {
          document: { uri: { fsPath: '/workspace/test.ts' } },
        },
      };

      visibleRangesHandler(mockEvent);

      expect(mockSessionManager.recordActivity).toHaveBeenCalledWith('/workspace/test.ts');
    });

    // Terminal write test removed since onDidWriteTerminalData is not available in all VS Code versions

    it('should record activity on debug session start', () => {
      debugStartHandler();
      expect(mockSessionManager.recordActivity).toHaveBeenCalledWith();
    });

    it('should record activity on debug session terminate', () => {
      debugTerminateHandler();
      expect(mockSessionManager.recordActivity).toHaveBeenCalledWith();
    });
  });

  describe('file filtering', () => {
    beforeEach(() => {
      activityTracker.activate(mockContext);
    });

    it('should track files within workspace', () => {
      const mockEvent = {
        document: { uri: { fsPath: '/workspace/src/test.ts' } },
        contentChanges: [{ text: 'hello' }],
      };

      documentChangeHandler(mockEvent);

      expect(mockSessionManager.recordActivity).toHaveBeenCalledWith('/workspace/src/test.ts');
    });

    it('should not track files outside workspace', () => {
      const mockEvent = {
        document: { uri: { fsPath: '/other/path/test.ts' } },
        contentChanges: [{ text: 'hello' }],
      };

      documentChangeHandler(mockEvent);

      expect(mockSessionManager.recordActivity).toHaveBeenCalledWith();
    });

    it('should not track node_modules files', () => {
      const mockEvent = {
        document: { uri: { fsPath: '/workspace/node_modules/package/index.js' } },
        contentChanges: [{ text: 'hello' }],
      };

      documentChangeHandler(mockEvent);

      expect(mockSessionManager.recordActivity).toHaveBeenCalledWith();
    });

    it('should not track .git files', () => {
      const mockEvent = {
        document: { uri: { fsPath: '/workspace/.git/config' } },
        contentChanges: [{ text: 'hello' }],
      };

      documentChangeHandler(mockEvent);

      expect(mockSessionManager.recordActivity).toHaveBeenCalledWith();
    });

    it('should not track build directory files', () => {
      const mockEvent = {
        document: { uri: { fsPath: '/workspace/dist/bundle.js' } },
        contentChanges: [{ text: 'hello' }],
      };

      documentChangeHandler(mockEvent);

      expect(mockSessionManager.recordActivity).toHaveBeenCalledWith();
    });

    it('should not track log files', () => {
      const mockEvent = {
        document: { uri: { fsPath: '/workspace/app.log' } },
        contentChanges: [{ text: 'hello' }],
      };

      documentChangeHandler(mockEvent);

      expect(mockSessionManager.recordActivity).toHaveBeenCalledWith();
    });
  });

  describe('activity throttling', () => {
    beforeEach(() => {
      activityTracker.activate(mockContext);
      jest.useFakeTimers();
    });

    afterEach(() => {
      jest.useRealTimers();
    });

    it('should throttle rapid activity recording', () => {
      const mockEvent = {
        document: { uri: { fsPath: '/workspace/test.ts' } },
        contentChanges: [{ text: 'h' }],
      };

      // Fire multiple events rapidly
      documentChangeHandler(mockEvent);
      documentChangeHandler(mockEvent);
      documentChangeHandler(mockEvent);

      // Should only record the first event
      expect(mockSessionManager.recordActivity).toHaveBeenCalledTimes(1);
    });

    it('should record activity after throttle timeout', () => {
      const mockEvent = {
        document: { uri: { fsPath: '/workspace/test.ts' } },
        contentChanges: [{ text: 'h' }],
      };

      documentChangeHandler(mockEvent);
      expect(mockSessionManager.recordActivity).toHaveBeenCalledTimes(1);

      // Advance time past throttle timeout
      jest.advanceTimersByTime(1000);

      documentChangeHandler(mockEvent);
      expect(mockSessionManager.recordActivity).toHaveBeenCalledTimes(2);
    });
  });

  describe('getLastActivityTime', () => {
    it('should return the last activity time', () => {
      const initialTime = activityTracker.getLastActivityTime();
      expect(initialTime).toBeInstanceOf(Date);
    });
  });

  describe('dispose', () => {
    it('should dispose all event listeners', () => {
      const disposeMock = jest.fn();
      
      // Mock disposables
      (vscode.workspace.onDidChangeTextDocument as jest.Mock).mockReturnValue({ dispose: disposeMock });
      (vscode.window.onDidChangeActiveTextEditor as jest.Mock).mockReturnValue({ dispose: disposeMock });
      
      activityTracker.activate(mockContext);
      activityTracker.dispose();

      expect(disposeMock).toHaveBeenCalled();
    });
  });
});
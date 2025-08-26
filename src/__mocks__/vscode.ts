export const window = {
  createStatusBarItem: jest.fn(() => ({
    text: '',
    tooltip: '',
    command: '',
    show: jest.fn(),
    hide: jest.fn(),
    dispose: jest.fn(),
  })),
  showWarningMessage: jest.fn(),
  showInformationMessage: jest.fn(),
  onDidChangeActiveTextEditor: jest.fn(() => ({ dispose: jest.fn() })),
  onDidChangeWindowState: jest.fn(() => ({ dispose: jest.fn() })),
  onDidChangeTextEditorSelection: jest.fn(() => ({ dispose: jest.fn() })),
  onDidChangeTextEditorVisibleRanges: jest.fn(() => ({ dispose: jest.fn() })),
};

export const workspace = {
  getConfiguration: jest.fn(() => ({
    get: jest.fn((key: string, defaultValue: any) => {
      const config: Record<string, any> = {
        'idleTimeoutMinutes': 5,
        'showNotifications': true,
        'autoStartSession': false,
      };
      return config[key] ?? defaultValue;
    }),
  })),
  onDidChangeTextDocument: jest.fn(() => ({ dispose: jest.fn() })),
  workspaceFolders: [
    { uri: { fsPath: '/workspace' } }
  ],
};

export const debug = {
  onDidStartDebugSession: jest.fn(() => ({ dispose: jest.fn() })),
  onDidTerminateDebugSession: jest.fn(() => ({ dispose: jest.fn() })),
};

export const StatusBarAlignment = {
  Left: 1,
  Right: 2,
};

export const commands = {
  registerCommand: jest.fn(() => ({ dispose: jest.fn() })),
};

export default {
  window,
  workspace,
  debug,
  StatusBarAlignment,
  commands,
};
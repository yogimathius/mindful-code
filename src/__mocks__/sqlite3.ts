export class Database {
  constructor(path: string) {
    // Mock constructor
  }

  serialize(callback: () => void): void {
    callback();
  }

  run(query: string, params?: any[], callback?: (err: Error | null, result?: any) => void): void {
    if (callback) {
      // Simulate successful execution
      callback(null, { changes: 1, lastID: 1 });
    }
  }

  get(query: string, params: any[], callback: (err: Error | null, row?: any) => void): void {
    // Mock get - return null for no results
    callback(null, null);
  }

  all(query: string, params: any[], callback: (err: Error | null, rows: any[]) => void): void {
    // Mock all - return empty array
    callback(null, []);
  }

  close(callback?: (err: Error | null) => void): void {
    if (callback) {
      callback(null);
    }
  }
}

export default { Database };
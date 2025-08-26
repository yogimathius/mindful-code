import * as path from 'path';
import * as fs from 'fs';
import { Database } from 'sqlite3';
import { SessionData } from '../models/Session';

export class DatabaseService {
  private db: Database;
  private dbPath: string;

  constructor(storagePath: string) {
    // Ensure storage directory exists
    if (!fs.existsSync(storagePath)) {
      fs.mkdirSync(storagePath, { recursive: true });
    }

    this.dbPath = path.join(storagePath, 'mindful-code.db');
    this.db = new Database(this.dbPath);
    
    this.initializeDatabase();
  }

  private initializeDatabase(): void {
    const createTableQuery = `
      CREATE TABLE IF NOT EXISTS sessions (
        id TEXT PRIMARY KEY,
        startTime TEXT NOT NULL,
        endTime TEXT,
        duration INTEGER NOT NULL,
        isActive INTEGER NOT NULL,
        isPaused INTEGER NOT NULL,
        pausedDuration INTEGER NOT NULL,
        filesWorkedOn TEXT NOT NULL,
        keystrokes INTEGER NOT NULL,
        activeTime INTEGER NOT NULL,
        flowStateDetected INTEGER NOT NULL,
        flowStateDuration INTEGER NOT NULL,
        interruptions INTEGER NOT NULL,
        createdAt TEXT NOT NULL,
        updatedAt TEXT NOT NULL
      )
    `;

    this.db.serialize(() => {
      this.db.run(createTableQuery, (err) => {
        if (err) {
          console.error('Error creating sessions table:', err);
        } else {
          console.log('Sessions table ready');
        }
      });

      // Create indexes for better query performance
      this.db.run(
        'CREATE INDEX IF NOT EXISTS idx_sessions_startTime ON sessions(startTime)'
      );
      this.db.run(
        'CREATE INDEX IF NOT EXISTS idx_sessions_endTime ON sessions(endTime)'
      );
      this.db.run(
        'CREATE INDEX IF NOT EXISTS idx_sessions_createdAt ON sessions(createdAt)'
      );
    });
  }

  async saveSession(sessionData: SessionData): Promise<void> {
    return new Promise((resolve, reject) => {
      const now = new Date().toISOString();
      const query = `
        INSERT OR REPLACE INTO sessions (
          id, startTime, endTime, duration, isActive, isPaused,
          pausedDuration, filesWorkedOn, keystrokes, activeTime,
          flowStateDetected, flowStateDuration, interruptions,
          createdAt, updatedAt
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
      `;

      const params = [
        sessionData.id,
        sessionData.startTime.toISOString(),
        sessionData.endTime?.toISOString() || null,
        sessionData.duration,
        sessionData.isActive ? 1 : 0,
        sessionData.isPaused ? 1 : 0,
        sessionData.pausedDuration,
        JSON.stringify(sessionData.filesWorkedOn),
        sessionData.keystrokes,
        sessionData.activeTime,
        sessionData.flowStateDetected ? 1 : 0,
        sessionData.flowStateDuration,
        sessionData.interruptions,
        now,
        now,
      ];

      this.db.run(query, params, function (err) {
        if (err) {
          console.error('Error saving session:', err);
          reject(err);
        } else {
          resolve();
        }
      });
    });
  }

  async getSession(sessionId: string): Promise<SessionData | null> {
    return new Promise((resolve, reject) => {
      const query = 'SELECT * FROM sessions WHERE id = ?';
      
      this.db.get(query, [sessionId], (err, row: any) => {
        if (err) {
          console.error('Error retrieving session:', err);
          reject(err);
        } else if (row) {
          resolve(this.mapRowToSessionData(row));
        } else {
          resolve(null);
        }
      });
    });
  }

  async getRecentSessions(days: number = 30): Promise<SessionData[]> {
    return new Promise((resolve, reject) => {
      const cutoffDate = new Date();
      cutoffDate.setDate(cutoffDate.getDate() - days);
      
      const query = `
        SELECT * FROM sessions 
        WHERE startTime >= ? 
        ORDER BY startTime DESC
      `;
      
      this.db.all(query, [cutoffDate.toISOString()], (err, rows: any[]) => {
        if (err) {
          console.error('Error retrieving recent sessions:', err);
          reject(err);
        } else {
          const sessions = rows.map(row => this.mapRowToSessionData(row));
          resolve(sessions);
        }
      });
    });
  }

  async getSessionStats(days: number = 7): Promise<{
    totalSessions: number;
    totalCodingTime: number;
    totalFlowTime: number;
    averageSessionLength: number;
    totalKeystrokes: number;
    uniqueFilesWorked: number;
  }> {
    return new Promise((resolve, reject) => {
      const cutoffDate = new Date();
      cutoffDate.setDate(cutoffDate.getDate() - days);
      
      const query = `
        SELECT 
          COUNT(*) as totalSessions,
          SUM(duration) as totalCodingTime,
          SUM(flowStateDuration) as totalFlowTime,
          AVG(duration) as averageSessionLength,
          SUM(keystrokes) as totalKeystrokes,
          GROUP_CONCAT(filesWorkedOn) as allFiles
        FROM sessions 
        WHERE startTime >= ? AND endTime IS NOT NULL
      `;
      
      this.db.get(query, [cutoffDate.toISOString()], (err, row: any) => {
        if (err) {
          console.error('Error calculating session stats:', err);
          reject(err);
        } else {
          // Calculate unique files
          const allFiles = new Set<string>();
          if (row.allFiles) {
            const fileArrays = row.allFiles.split(',').map((files: string) => {
              try {
                return JSON.parse(files);
              } catch {
                return [];
              }
            });
            fileArrays.forEach((files: string[]) => {
              files.forEach(file => allFiles.add(file));
            });
          }

          resolve({
            totalSessions: row.totalSessions || 0,
            totalCodingTime: row.totalCodingTime || 0,
            totalFlowTime: row.totalFlowTime || 0,
            averageSessionLength: row.averageSessionLength || 0,
            totalKeystrokes: row.totalKeystrokes || 0,
            uniqueFilesWorked: allFiles.size,
          });
        }
      });
    });
  }

  async deleteSession(sessionId: string): Promise<boolean> {
    return new Promise((resolve, reject) => {
      const query = 'DELETE FROM sessions WHERE id = ?';
      
      this.db.run(query, [sessionId], function (err) {
        if (err) {
          console.error('Error deleting session:', err);
          reject(err);
        } else {
          resolve(this.changes > 0);
        }
      });
    });
  }

  async deleteOldSessions(daysToKeep: number = 90): Promise<number> {
    return new Promise((resolve, reject) => {
      const cutoffDate = new Date();
      cutoffDate.setDate(cutoffDate.getDate() - daysToKeep);
      
      const query = 'DELETE FROM sessions WHERE startTime < ?';
      
      this.db.run(query, [cutoffDate.toISOString()], function (err) {
        if (err) {
          console.error('Error deleting old sessions:', err);
          reject(err);
        } else {
          console.log(`Deleted ${this.changes} old sessions`);
          resolve(this.changes);
        }
      });
    });
  }

  private mapRowToSessionData(row: any): SessionData {
    return {
      id: row.id,
      startTime: new Date(row.startTime),
      endTime: row.endTime ? new Date(row.endTime) : undefined,
      duration: row.duration,
      isActive: Boolean(row.isActive),
      isPaused: Boolean(row.isPaused),
      pausedDuration: row.pausedDuration,
      filesWorkedOn: JSON.parse(row.filesWorkedOn),
      keystrokes: row.keystrokes,
      activeTime: row.activeTime,
      flowStateDetected: Boolean(row.flowStateDetected),
      flowStateDuration: row.flowStateDuration,
      interruptions: row.interruptions,
    };
  }

  close(): void {
    this.db.close((err) => {
      if (err) {
        console.error('Error closing database:', err);
      } else {
        console.log('Database connection closed');
      }
    });
  }
}
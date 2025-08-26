import { SessionData, DashboardStats } from '../types';
import { format, subDays, addMinutes } from 'date-fns';

export function mockSessionData(): SessionData[] {
  const sessions: SessionData[] = [];
  const now = new Date();
  
  // Generate 30 days of mock data
  for (let i = 0; i < 30; i++) {
    const dayStart = subDays(now, i);
    const sessionsToday = Math.floor(Math.random() * 4) + 1; // 1-4 sessions per day
    
    for (let j = 0; j < sessionsToday; j++) {
      const startTime = new Date(dayStart);
      startTime.setHours(9 + Math.floor(Math.random() * 8)); // 9 AM to 5 PM
      startTime.setMinutes(Math.floor(Math.random() * 60));
      
      const duration = (20 + Math.floor(Math.random() * 60)) * 60 * 1000; // 20-80 minutes
      const endTime = addMinutes(startTime, duration / 60000);
      
      const activeTime = duration * (0.7 + Math.random() * 0.25); // 70-95% active
      const keystrokes = Math.floor(duration / 1000 * (1 + Math.random() * 3)); // Variable typing speed
      const flowStateDuration = Math.random() > 0.6 ? duration * (0.3 + Math.random() * 0.4) : 0;
      
      const files = [
        'src/components/Dashboard.tsx',
        'src/services/SessionManager.ts',
        'src/utils/helpers.ts',
        'src/types/index.ts',
        'README.md',
        'package.json'
      ];
      
      const filesWorkedOn = files
        .sort(() => 0.5 - Math.random())
        .slice(0, Math.floor(Math.random() * 4) + 1);
      
      sessions.push({
        id: `session_${i}_${j}_${Date.now()}`,
        startTime,
        endTime,
        duration,
        isActive: false,
        isPaused: false,
        pausedDuration: Math.floor(Math.random() * 300000), // 0-5 min
        filesWorkedOn,
        keystrokes,
        activeTime,
        flowStateDetected: flowStateDuration > 0,
        flowStateDuration,
        interruptions: Math.floor(Math.random() * 3),
      });
    }
  }
  
  return sessions.sort((a, b) => b.startTime.getTime() - a.startTime.getTime());
}

export function calculateDashboardStats(sessions: SessionData[]): DashboardStats {
  const totalSessions = sessions.length;
  const totalCodingTime = sessions.reduce((sum, s) => sum + s.duration, 0);
  const totalFlowTime = sessions.reduce((sum, s) => sum + s.flowStateDuration, 0);
  const averageSessionLength = totalCodingTime / totalSessions;
  
  // Calculate productivity score (0-100)
  const avgActiveRatio = sessions.reduce((sum, s) => sum + (s.activeTime / s.duration), 0) / sessions.length;
  const avgFlowRatio = totalFlowTime / totalCodingTime;
  const productivityScore = Math.round((avgActiveRatio * 0.6 + avgFlowRatio * 0.4) * 100);
  
  // Find peak hours
  const hourCounts = new Array(24).fill(0);
  sessions.forEach(session => {
    hourCounts[session.startTime.getHours()]++;
  });
  const peakHours = hourCounts
    .map((count, hour) => ({ hour, count }))
    .sort((a, b) => b.count - a.count)
    .slice(0, 3)
    .map(h => h.hour);
  
  // Most productive days
  const dayStats = new Map<string, { sessions: number; time: number }>();
  sessions.forEach(session => {
    const day = format(session.startTime, 'EEEE');
    const current = dayStats.get(day) || { sessions: 0, time: 0 };
    dayStats.set(day, {
      sessions: current.sessions + 1,
      time: current.time + session.duration
    });
  });
  
  const mostProductiveDays = Array.from(dayStats.entries())
    .sort(([,a], [,b]) => b.time - a.time)
    .slice(0, 3)
    .map(([day]) => day);
  
  // Weekly trend (simplified)
  const weeklyTrend: 'up' | 'down' | 'stable' = Math.random() > 0.6 ? 'up' : Math.random() > 0.3 ? 'stable' : 'down';
  
  // Daily stats for charts
  const dailyStats = [];
  const today = new Date();
  for (let i = 6; i >= 0; i--) {
    const date = subDays(today, i);
    const dateStr = format(date, 'MMM dd');
    const daySessions = sessions.filter(s => 
      format(s.startTime, 'yyyy-MM-dd') === format(date, 'yyyy-MM-dd')
    );
    
    dailyStats.push({
      date: dateStr,
      sessions: daySessions.length,
      codingTime: daySessions.reduce((sum, s) => sum + s.duration, 0) / 3600000, // Convert to hours
      flowTime: daySessions.reduce((sum, s) => sum + s.flowStateDuration, 0) / 3600000,
    });
  }
  
  return {
    totalSessions,
    totalCodingTime,
    totalFlowTime,
    averageSessionLength,
    productivityScore,
    peakHours,
    mostProductiveDays,
    weeklyTrend,
    dailyStats,
  };
}
import { Clock, TrendingUp, Zap, FileText } from 'lucide-react';
import { BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, LineChart, Line } from 'recharts';
import { DashboardStats, SessionData } from '../types';
import { format } from 'date-fns';

interface DashboardProps {
  stats: DashboardStats;
  recentSessions: SessionData[];
}

function Dashboard({ stats, recentSessions }: DashboardProps) {
  const formatDuration = (ms: number) => {
    const hours = Math.floor(ms / 3600000);
    const minutes = Math.floor((ms % 3600000) / 60000);
    return hours > 0 ? `${hours}h ${minutes}m` : `${minutes}m`;
  };

  const formatHour = (hour: number) => {
    return hour === 0 ? '12 AM' : hour <= 12 ? `${hour} AM` : `${hour - 12} PM`;
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex justify-between items-start">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">Dashboard</h1>
          <p className="text-gray-600">Your coding productivity insights</p>
        </div>
        
        <div className="flex items-center space-x-2">
          <div className={`flex items-center px-3 py-1 rounded-full text-sm ${
            stats.weeklyTrend === 'up' 
              ? 'bg-green-100 text-green-800' 
              : stats.weeklyTrend === 'down'
              ? 'bg-red-100 text-red-800'
              : 'bg-gray-100 text-gray-800'
          }`}>
            <TrendingUp className="h-3 w-3 mr-1" />
            {stats.weeklyTrend === 'up' ? 'Trending Up' : stats.weeklyTrend === 'down' ? 'Trending Down' : 'Stable'}
          </div>
        </div>
      </div>

      {/* Stats Cards */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        <div className="card">
          <div className="flex items-center">
            <div className="flex-shrink-0">
              <Clock className="h-8 w-8 text-blue-600" />
            </div>
            <div className="ml-4">
              <p className="text-sm font-medium text-gray-500">Total Coding Time</p>
              <p className="text-2xl font-semibold text-gray-900">
                {formatDuration(stats.totalCodingTime)}
              </p>
            </div>
          </div>
        </div>

        <div className="card">
          <div className="flex items-center">
            <div className="flex-shrink-0">
              <Zap className="h-8 w-8 text-green-600" />
            </div>
            <div className="ml-4">
              <p className="text-sm font-medium text-gray-500">Flow State Time</p>
              <p className="text-2xl font-semibold text-gray-900">
                {formatDuration(stats.totalFlowTime)}
              </p>
            </div>
          </div>
        </div>

        <div className="card">
          <div className="flex items-center">
            <div className="flex-shrink-0">
              <FileText className="h-8 w-8 text-purple-600" />
            </div>
            <div className="ml-4">
              <p className="text-sm font-medium text-gray-500">Sessions</p>
              <p className="text-2xl font-semibold text-gray-900">
                {stats.totalSessions}
              </p>
            </div>
          </div>
        </div>

        <div className="card">
          <div className="flex items-center">
            <div className="flex-shrink-0">
              <TrendingUp className="h-8 w-8 text-orange-600" />
            </div>
            <div className="ml-4">
              <p className="text-sm font-medium text-gray-500">Productivity Score</p>
              <p className="text-2xl font-semibold text-gray-900">
                {stats.productivityScore}%
              </p>
            </div>
          </div>
        </div>
      </div>

      {/* Charts */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Daily Activity Chart */}
        <div className="card">
          <h3 className="text-lg font-semibold text-gray-900 mb-4">Daily Activity (Last 7 Days)</h3>
          <ResponsiveContainer width="100%" height={300}>
            <BarChart data={stats.dailyStats}>
              <CartesianGrid strokeDasharray="3 3" />
              <XAxis dataKey="date" />
              <YAxis />
              <Tooltip 
                formatter={(value: number, name: string) => [
                  `${value.toFixed(1)}h`, 
                  name === 'codingTime' ? 'Coding Time' : 'Flow Time'
                ]}
              />
              <Bar dataKey="codingTime" fill="#3B82F6" name="codingTime" />
              <Bar dataKey="flowTime" fill="#10B981" name="flowTime" />
            </BarChart>
          </ResponsiveContainer>
        </div>

        {/* Session Trend */}
        <div className="card">
          <h3 className="text-lg font-semibold text-gray-900 mb-4">Session Count Trend</h3>
          <ResponsiveContainer width="100%" height={300}>
            <LineChart data={stats.dailyStats}>
              <CartesianGrid strokeDasharray="3 3" />
              <XAxis dataKey="date" />
              <YAxis />
              <Tooltip formatter={(value: number) => [`${value} sessions`, 'Sessions']} />
              <Line 
                type="monotone" 
                dataKey="sessions" 
                stroke="#8B5CF6" 
                strokeWidth={2}
                dot={{ fill: '#8B5CF6' }}
              />
            </LineChart>
          </ResponsiveContainer>
        </div>
      </div>

      {/* Insights and Recent Sessions */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Insights */}
        <div className="card">
          <h3 className="text-lg font-semibold text-gray-900 mb-4">Your Insights</h3>
          <div className="space-y-4">
            <div className="flex items-start space-x-3">
              <div className="flex-shrink-0 w-2 h-2 bg-blue-600 rounded-full mt-2"></div>
              <div>
                <p className="font-medium text-gray-900">Peak Hours</p>
                <p className="text-sm text-gray-600">
                  Most productive at {stats.peakHours.map(formatHour).join(', ')}
                </p>
              </div>
            </div>
            
            <div className="flex items-start space-x-3">
              <div className="flex-shrink-0 w-2 h-2 bg-green-600 rounded-full mt-2"></div>
              <div>
                <p className="font-medium text-gray-900">Best Days</p>
                <p className="text-sm text-gray-600">
                  {stats.mostProductiveDays.join(', ')}
                </p>
              </div>
            </div>
            
            <div className="flex items-start space-x-3">
              <div className="flex-shrink-0 w-2 h-2 bg-purple-600 rounded-full mt-2"></div>
              <div>
                <p className="font-medium text-gray-900">Average Session</p>
                <p className="text-sm text-gray-600">
                  {formatDuration(stats.averageSessionLength)} per session
                </p>
              </div>
            </div>
          </div>
        </div>

        {/* Recent Sessions */}
        <div className="card">
          <h3 className="text-lg font-semibold text-gray-900 mb-4">Recent Sessions</h3>
          <div className="space-y-3">
            {recentSessions.map((session) => (
              <div key={session.id} className="flex items-center justify-between py-2 border-b border-gray-100 last:border-0">
                <div>
                  <p className="text-sm font-medium text-gray-900">
                    {format(session.startTime, 'MMM dd, HH:mm')}
                  </p>
                  <p className="text-xs text-gray-500">
                    {session.filesWorkedOn.length} files • {session.keystrokes} keystrokes
                  </p>
                </div>
                <div className="text-right">
                  <p className="text-sm font-medium text-gray-900">
                    {formatDuration(session.duration)}
                  </p>
                  {session.flowStateDetected && (
                    <div className="flex items-center justify-end">
                      <Zap className="h-3 w-3 text-green-600" />
                      <span className="text-xs text-green-600 ml-1">Flow</span>
                    </div>
                  )}
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}

export default Dashboard;
import { useState, useEffect } from 'react';
import { Clock, Zap, BarChart3, Settings as SettingsIcon } from 'lucide-react';
import Dashboard from './components/Dashboard';
import { SessionData, DashboardStats } from './types';
import { mockSessionData, calculateDashboardStats } from './utils/mockData';

type ActiveTab = 'dashboard' | 'sessions' | 'flow' | 'settings';

function App() {
  const [activeTab, setActiveTab] = useState<ActiveTab>('dashboard');
  const [sessions, setSessions] = useState<SessionData[]>([]);
  const [stats, setStats] = useState<DashboardStats | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    // Simulate loading data
    setTimeout(() => {
      const mockSessions = mockSessionData();
      setSessions(mockSessions);
      setStats(calculateDashboardStats(mockSessions));
      setLoading(false);
    }, 1000);
  }, []);

  const tabs = [
    { id: 'dashboard', name: 'Dashboard', icon: BarChart3 },
    { id: 'sessions', name: 'Sessions', icon: Clock },
    { id: 'flow', name: 'Flow Analytics', icon: Zap },
    { id: 'settings', name: 'Settings', icon: SettingsIcon },
  ];

  if (loading) {
    return (
      <div className="min-h-screen bg-gray-50 flex items-center justify-center">
        <div className="text-center">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600 mx-auto"></div>
          <p className="mt-4 text-gray-600">Loading your coding insights...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gray-50">
      {/* Header */}
      <header className="bg-white shadow-sm border-b">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex justify-between items-center h-16">
            <div className="flex items-center">
              <Zap className="h-8 w-8 text-blue-600" />
              <h1 className="ml-3 text-xl font-bold text-gray-900">Mindful Code</h1>
            </div>
            
            <nav className="flex space-x-8">
              {tabs.map((tab) => {
                const Icon = tab.icon;
                return (
                  <button
                    key={tab.id}
                    onClick={() => setActiveTab(tab.id as ActiveTab)}
                    className={`flex items-center px-3 py-2 text-sm font-medium rounded-md transition-colors ${
                      activeTab === tab.id
                        ? 'text-blue-600 bg-blue-50'
                        : 'text-gray-600 hover:text-gray-900 hover:bg-gray-50'
                    }`}
                  >
                    <Icon className="h-4 w-4 mr-2" />
                    {tab.name}
                  </button>
                );
              })}
            </nav>
          </div>
        </div>
      </header>

      {/* Main Content */}
      <main className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        {activeTab === 'dashboard' && stats && (
          <Dashboard stats={stats} recentSessions={sessions.slice(0, 5)} />
        )}
        
        {activeTab === 'sessions' && (
          <div className="text-center py-12">
            <p className="text-gray-500">Session list coming soon</p>
          </div>
        )}
        
        {activeTab === 'flow' && (
          <div className="text-center py-12">
            <p className="text-gray-500">Flow analytics coming soon</p>
          </div>
        )}
        
        {activeTab === 'settings' && (
          <div className="text-center py-12">
            <p className="text-gray-500">Settings coming soon</p>
          </div>
        )}
      </main>
    </div>
  );
}

export default App;
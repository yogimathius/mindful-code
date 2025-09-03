-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Users and privacy settings
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    subscription_tier VARCHAR(20) DEFAULT 'free',
    privacy_settings JSONB DEFAULT '{}',
    timezone VARCHAR(50) DEFAULT 'UTC',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create index for faster user lookups
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_created_at ON users(created_at);

-- Coding sessions with detailed metrics
CREATE TABLE coding_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    start_time TIMESTAMP WITH TIME ZONE NOT NULL,
    end_time TIMESTAMP WITH TIME ZONE,
    total_duration_ms BIGINT,
    active_duration_ms BIGINT DEFAULT 0,
    files_modified INTEGER DEFAULT 0,
    keystrokes INTEGER DEFAULT 0,
    lines_added INTEGER DEFAULT 0,
    lines_deleted INTEGER DEFAULT 0,
    language_breakdown JSONB DEFAULT '{}',
    project_path VARCHAR(500),
    environment_data JSONB DEFAULT '{}',
    flow_state_periods JSONB DEFAULT '[]',
    interruption_count INTEGER DEFAULT 0,
    error_rate DECIMAL(5,4) DEFAULT 0,
    focus_score DECIMAL(3,2) DEFAULT 0,
    productivity_score DECIMAL(3,2) DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Performance indexes for sessions
CREATE INDEX idx_sessions_user_id ON coding_sessions(user_id);
CREATE INDEX idx_sessions_start_time ON coding_sessions(start_time);
CREATE INDEX idx_sessions_user_start_time ON coding_sessions(user_id, start_time);
CREATE INDEX idx_sessions_end_time ON coding_sessions(end_time) WHERE end_time IS NOT NULL;

-- Flow state detection data
CREATE TABLE flow_states (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL REFERENCES coding_sessions(id) ON DELETE CASCADE,
    start_time TIMESTAMP WITH TIME ZONE NOT NULL,
    end_time TIMESTAMP WITH TIME ZONE,
    duration_ms BIGINT,
    intensity_score DECIMAL(3,2) NOT NULL,
    typing_rhythm_data JSONB DEFAULT '{}',
    context_switches INTEGER DEFAULT 0,
    error_events JSONB DEFAULT '[]',
    recovery_time_ms BIGINT,
    environmental_factors JSONB DEFAULT '{}',
    ml_features JSONB DEFAULT '{}',
    confidence_score DECIMAL(3,2) DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Flow state indexes
CREATE INDEX idx_flow_states_session_id ON flow_states(session_id);
CREATE INDEX idx_flow_states_start_time ON flow_states(start_time);
CREATE INDEX idx_flow_states_intensity ON flow_states(intensity_score);

-- Team analytics (premium feature)
CREATE TABLE teams (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    owner_id UUID NOT NULL REFERENCES users(id),
    settings JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_teams_owner_id ON teams(owner_id);

CREATE TABLE team_members (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role VARCHAR(50) DEFAULT 'member',
    data_sharing_consent BOOLEAN DEFAULT false,
    joined_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(team_id, user_id)
);

CREATE INDEX idx_team_members_team_id ON team_members(team_id);
CREATE INDEX idx_team_members_user_id ON team_members(user_id);

-- Precomputed insights for performance
CREATE TABLE user_insights (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    insight_type VARCHAR(100) NOT NULL,
    insight_data JSONB NOT NULL,
    confidence_score DECIMAL(3,2),
    date_range_start DATE,
    date_range_end DATE,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_user_insights_user_id ON user_insights(user_id);
CREATE INDEX idx_user_insights_type ON user_insights(insight_type);
CREATE INDEX idx_user_insights_active ON user_insights(is_active) WHERE is_active = true;
CREATE INDEX idx_user_insights_date_range ON user_insights(date_range_start, date_range_end);

-- Encrypted user data (for GDPR compliance)
CREATE TABLE encrypted_user_data (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    data_type VARCHAR(100) NOT NULL,
    encrypted_data BYTEA NOT NULL,
    encryption_key_id VARCHAR(100) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_encrypted_data_user_id ON encrypted_user_data(user_id);
CREATE INDEX idx_encrypted_data_type ON encrypted_user_data(data_type);

-- Performance monitoring table
CREATE TABLE performance_metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    endpoint VARCHAR(255) NOT NULL,
    method VARCHAR(10) NOT NULL,
    response_time_ms INTEGER NOT NULL,
    status_code INTEGER NOT NULL,
    user_id UUID REFERENCES users(id),
    timestamp TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    additional_data JSONB DEFAULT '{}'
);

CREATE INDEX idx_performance_metrics_timestamp ON performance_metrics(timestamp);
CREATE INDEX idx_performance_metrics_endpoint ON performance_metrics(endpoint);

-- Function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Create triggers for updated_at
CREATE TRIGGER update_users_updated_at BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_coding_sessions_updated_at BEFORE UPDATE ON coding_sessions
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_teams_updated_at BEFORE UPDATE ON teams
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_user_insights_updated_at BEFORE UPDATE ON user_insights
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
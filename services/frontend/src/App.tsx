import React from 'react';
import './App.css';

interface HealthStatus {
  status: string;
  service: string;
  version: string;
}

function App() {
  const [health] = React.useState<HealthStatus>({
    status: 'healthy',
    service: 'frontend',
    version: '0.1.0'
  });

  return (
    <div className="App">
      <header className="App-header">
        <h1>HermesFlow</h1>
        <p>Advanced Trading Platform</p>
        <div className="health-status">
          <p>Status: {health.status}</p>
          <p>Service: {health.service}</p>
          <p>Version: {health.version}</p>
        </div>
      </header>
    </div>
  );
}

export default App;

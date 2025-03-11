import { useState, useEffect } from 'react';

interface NodeStatus {
  id: string;
  name: string;
  status: 'online' | 'offline' | 'warning';
  lastSeen: string;
  version: string;
}

const Dashboard = () => {
  const [nodes, setNodes] = useState<NodeStatus[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    // In a real application, we would fetch this data from an API
    const fetchNodes = async () => {
      try {
        // Simulate API call
        await new Promise(resolve => setTimeout(resolve, 1000));
        
        // Mock data
        const mockNodes: NodeStatus[] = [
          { id: '1', name: 'Node Alpha', status: 'online', lastSeen: new Date().toISOString(), version: '1.0.0' },
          { id: '2', name: 'Node Beta', status: 'online', lastSeen: new Date().toISOString(), version: '1.0.0' },
          { id: '3', name: 'Node Gamma', status: 'warning', lastSeen: new Date().toISOString(), version: '0.9.0' },
          { id: '4', name: 'Node Delta', status: 'offline', lastSeen: new Date(Date.now() - 86400000).toISOString(), version: '1.0.0' },
        ];
        
        setNodes(mockNodes);
        setIsLoading(false);
      } catch (err) {
        setError('Failed to fetch node data');
        setIsLoading(false);
      }
    };

    fetchNodes();
  }, []);

  if (isLoading) return <div className="loading">Loading dashboard data...</div>;
  if (error) return <div className="error">{error}</div>;

  return (
    <div className="page dashboard-page">
      <h1>Network Dashboard</h1>
      
      <div className="dashboard-summary">
        <div className="dashboard-card">
          <h3>Total Nodes</h3>
          <p className="dashboard-value">{nodes.length}</p>
        </div>
        <div className="dashboard-card">
          <h3>Online Nodes</h3>
          <p className="dashboard-value">{nodes.filter(node => node.status === 'online').length}</p>
        </div>
        <div className="dashboard-card">
          <h3>Warning Nodes</h3>
          <p className="dashboard-value">{nodes.filter(node => node.status === 'warning').length}</p>
        </div>
        <div className="dashboard-card">
          <h3>Offline Nodes</h3>
          <p className="dashboard-value">{nodes.filter(node => node.status === 'offline').length}</p>
        </div>
      </div>
      
      <h2>Node Status</h2>
      <div className="node-table-container">
        <table className="node-table">
          <thead>
            <tr>
              <th>Name</th>
              <th>Status</th>
              <th>Last Seen</th>
              <th>Version</th>
            </tr>
          </thead>
          <tbody>
            {nodes.map(node => (
              <tr key={node.id} className={`node-row status-${node.status}`}>
                <td>{node.name}</td>
                <td>
                  <span className={`status-indicator ${node.status}`}></span>
                  {node.status.charAt(0).toUpperCase() + node.status.slice(1)}
                </td>
                <td>{new Date(node.lastSeen).toLocaleString()}</td>
                <td>{node.version}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
};

export default Dashboard;
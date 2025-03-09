import React from 'react';
import { BrowserRouter as Router, Routes, Route } from 'react-router-dom';
import { NodeDashboard } from './components/nodes/NodeDashboard';
import { Layout } from './components/layout/Layout';

function App() {
  return (
    <Router>
      <Layout>
        <Routes>
          <Route path="/" element={<NodeDashboard />} />
          {/* Add other routes as they are implemented */}
        </Routes>
      </Layout>
    </Router>
  );
}

export default App;
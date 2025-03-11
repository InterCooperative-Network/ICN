import React from 'react';
import Link from 'next/link';
import Head from 'next/head';

const Dashboard: React.FC = () => {
  return (
    <div className="min-h-screen bg-gray-100">
      <Head>
        <title>ICN Dashboard</title>
        <meta name="description" content="Inter-Cooperative Network Dashboard" />
      </Head>
      
      <header className="bg-blue-600 text-white shadow-md">
        <div className="container mx-auto py-4 px-6">
          <h1 className="text-3xl font-bold">ICN Dashboard</h1>
          <p className="text-sm opacity-80">Inter-Cooperative Network Management</p>
        </div>
      </header>
      
      <main className="container mx-auto py-8 px-6">
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {/* Federation Management Card */}
          <div className="bg-white rounded-lg shadow-md p-6 hover:shadow-lg transition-shadow">
            <h2 className="text-xl font-semibold mb-4 text-blue-700">Federation Management</h2>
            <p className="text-gray-600 mb-4">Manage federation membership, settings, and communications.</p>
            <div className="mt-auto">
              <Link href="/federation" className="text-blue-500 hover:text-blue-700 font-medium">
                Open Federation Panel →
              </Link>
            </div>
          </div>
          
          {/* Governance Card */}
          <div className="bg-white rounded-lg shadow-md p-6 hover:shadow-lg transition-shadow">
            <h2 className="text-xl font-semibold mb-4 text-blue-700">Governance</h2>
            <p className="text-gray-600 mb-4">Participate in governance activities, vote on proposals, and track decisions.</p>
            <div className="mt-auto">
              <Link href="/governance" className="text-blue-500 hover:text-blue-700 font-medium">
                Open Governance Panel →
              </Link>
            </div>
          </div>
          
          {/* Resource Allocation Card */}
          <div className="bg-white rounded-lg shadow-md p-6 hover:shadow-lg transition-shadow">
            <h2 className="text-xl font-semibold mb-4 text-blue-700">Resource Allocation</h2>
            <p className="text-gray-600 mb-4">Monitor and manage resource allocation across the network.</p>
            <div className="mt-auto">
              <Link href="/resources" className="text-blue-500 hover:text-blue-700 font-medium">
                Open Resource Panel →
              </Link>
            </div>
          </div>
          
          {/* Consensus Status Card */}
          <div className="bg-white rounded-lg shadow-md p-6 hover:shadow-lg transition-shadow">
            <h2 className="text-xl font-semibold mb-4 text-blue-700">Consensus Status</h2>
            <p className="text-gray-600 mb-4">View consensus status, validator information, and network health.</p>
            <div className="mt-auto">
              <Link href="/consensus" className="text-blue-500 hover:text-blue-700 font-medium">
                Open Consensus Panel →
              </Link>
            </div>
          </div>
          
          {/* Identity Management Card */}
          <div className="bg-white rounded-lg shadow-md p-6 hover:shadow-lg transition-shadow">
            <h2 className="text-xl font-semibold mb-4 text-blue-700">Identity Management</h2>
            <p className="text-gray-600 mb-4">Manage your identity, keys, and permissions within the network.</p>
            <div className="mt-auto">
              <Link href="/identity" className="text-blue-500 hover:text-blue-700 font-medium">
                Open Identity Panel →
              </Link>
            </div>
          </div>
          
          {/* Network Statistics Card */}
          <div className="bg-white rounded-lg shadow-md p-6 hover:shadow-lg transition-shadow">
            <h2 className="text-xl font-semibold mb-4 text-blue-700">Network Statistics</h2>
            <p className="text-gray-600 mb-4">View network performance metrics and system health.</p>
            <div className="mt-auto">
              <Link href="/stats" className="text-blue-500 hover:text-blue-700 font-medium">
                Open Statistics Panel →
              </Link>
            </div>
          </div>
        </div>
      </main>
      
      <footer className="bg-gray-800 text-white mt-auto">
        <div className="container mx-auto py-4 px-6">
          <p className="text-sm">Inter-Cooperative Network © {new Date().getFullYear()}</p>
        </div>
      </footer>
    </div>
  );
};

export default Dashboard;
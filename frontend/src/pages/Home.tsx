const Home = () => {
  return (
    <div className="page home-page">
      <h1>Welcome to the Inter-Cooperative Network</h1>
      <p>
        A decentralized platform for secure federation communications and cooperative resource sharing.
      </p>
      
      <section className="features-section">
        <h2>Key Features</h2>
        <div className="features-grid">
          <div className="feature-card">
            <h3>Secure Federation Communications</h3>
            <p>Uses the Secure Datagram Protocol (SDP) for reliable and secure inter-federation messaging.</p>
          </div>
          
          <div className="feature-card">
            <h3>Proof of Cooperation Consensus</h3>
            <p>Democratic validator selection with reputation-based incentives.</p>
          </div>
          
          <div className="feature-card">
            <h3>Zero-Knowledge Proofs</h3>
            <p>Privacy-preserving resource verification and governance.</p>
          </div>
          
          <div className="feature-card">
            <h3>Resource Federation</h3>
            <p>Dynamic resource pooling and allocation across federations.</p>
          </div>
        </div>
      </section>
    </div>
  );
};

export default Home; 
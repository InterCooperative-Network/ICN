import { Link } from 'react-router-dom';

const NotFound = () => {
  return (
    <div className="page not-found-page">
      <h1>404 - Page Not Found</h1>
      <p>The page you are looking for does not exist.</p>
      <Link to="/" className="btn btn-primary">
        Return to Home
      </Link>
    </div>
  );
};

export default NotFound; 
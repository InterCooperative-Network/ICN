class ICNError extends Error {
  constructor(code, message, details = {}) {
    super(message);
    this.code = code;
    this.details = details;
  }
}

const ErrorCodes = {
  GVRN_001: 'Governance proposal not found',
  AUTH_001: 'Invalid authentication',
  FED_001: 'Federation sync failed'
};

module.exports = { ICNError, ErrorCodes };

const winston = require('winston');

const logger = winston.createLogger({
  format: winston.format.json(),
  defaultMeta: { service: 'icn-service' },
  transports: [
    new winston.transports.Console(),
    new winston.transports.File({ filename: 'error.log', level: 'error' }),
    new winston.transports.File({ filename: 'combined.log' })
  ]
});

function logError(error, context) {
  logger.error({
    error_code: error.code || 'UNKNOWN',
    message: error.message,
    stack: error.stack,
    ...context
  });
}

module.exports = { logger, logError };

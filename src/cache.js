const NodeCache = require('node-cache');
const cache = new NodeCache({ stdTTL: 100, checkperiod: 120 });

function getCachedData(key, fetchFunction) {
  let value = cache.get(key);
  if (value == undefined) {
    value = fetchFunction();
    cache.set(key, value);
  }
  return value;
}

module.exports = { getCachedData };

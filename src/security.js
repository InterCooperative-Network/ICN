const bls = require('bls-lib');
const didRegistry = require('did-registry');

function verifySignature(message, signature, publicKey) {
  return bls.verify(message, signature, publicKey);
}

function registerDID(did, publicKey) {
  return didRegistry.register(did, publicKey);
}

module.exports = { verifySignature, registerDID };

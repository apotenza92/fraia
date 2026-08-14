const http = require('node:http');
const https = require('node:https');

function blocked() {
  throw new Error('network access is forbidden during the offline OCR fixture');
}

http.request = blocked;
http.get = blocked;
https.request = blocked;
https.get = blocked;
global.fetch = blocked;

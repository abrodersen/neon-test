#!/usr/bin/env node

const lib = require('./test-lib');
const fs = require('fs');

let buffer = new lib.WriteBuffer();

let file = fs.createReadStream('test.txt');

file.pipe(buffer);

file.on('end', () => {
  console.log('buffer size: ' + buffer.size());
});

console.log(Object.keys(lib));

lib.cb((resolve) => {
  let message = 'hello, continuation!';
  setImmediate(() => resolve(message));
}, (msg) => {
  console.log('got message: ' + msg);
});
  

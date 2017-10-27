const internal = require('../native');
const stream = require('stream');

class WriteBuffer extends stream.Writable {
    constructor(options) {
        super(options);

        this.buffer = new internal.WritableBuffer();
    }

    _write(chunk, encoding, callback) {
        this.buffer.write(chunk, encoding, callback);
    }

    size() {
        return this.buffer.size();
    }

}

module.exports.WriteBuffer = WriteBuffer;

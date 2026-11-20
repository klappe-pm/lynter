import {gzipSync} from 'fflate';

/** A tiny gzip+base64 payload shaped like the build output. */
export default Buffer.from(gzipSync(new Uint8Array([0, 97, 115, 109]))).toString('base64');

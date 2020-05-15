const native = process.dlopen(module, './target/debug/libwin_loop_poc.dylib')

exports.createWindow()

// there's no simple way (yet)
// so I'll probably just patch global timer-related functions
setInterval(() => console.log('tick'), 100)

// I/O is easier
// (nc localhost 8124 to awake UI thread)
// (example copied from nodejs docs)
const net = require('net');
const server = net.createServer((c) => {
  // 'connection' listener.
  console.log('client connected');
  c.on('end', () => {
    console.log('client disconnected');
  });
  c.write('hello\r\n');
  c.pipe(c);
});
server.on('error', (err) => {
  throw err;
});
server.listen(8124, () => {
  console.log('server bound');
});

const loop = () => {
    exports.waitEvent()
    setTimeout(loop, 100)
}

loop()

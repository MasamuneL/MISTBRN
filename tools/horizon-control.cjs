'use strict';
const fs = require('node:fs/promises');
const net = require('node:net');
let socket, done = false;
const privateValues = [];
const timer = setTimeout(() => finish({ error: 'timeout' }), 10000);
function finish(reply, status = 1) {
  if (done) return;
  done = true;
  clearTimeout(timer);
  socket?.destroy();
  let output = JSON.stringify(reply);
  for (const value of privateValues) {
    output = output.replaceAll(JSON.stringify(value).slice(1, -1), '[redacted]');
  }
  process.exitCode = status;
  process.stdout.write(output + '\n');
}
async function main() {
  const options = {};
  const args = process.argv.slice(2);
  for (let i = 0; i < args.length; i += 2) {
    const key = args[i];
    if (!['--socket', '--command-file'].includes(key) || !args[i + 1] || key in options) {
      return finish({ error: 'usage: --socket PATH --command-file PATH' });
    }
    options[key] = args[i + 1];
  }
  if (!options['--socket'] || !options['--command-file']) {
    return finish({ error: 'usage: --socket PATH --command-file PATH' });
  }
  privateValues.push(options['--socket']);
  const [endpointText, commandText] = await Promise.all([
    fs.readFile(options['--socket'], 'utf8'),
    fs.readFile(options['--command-file'], 'utf8'),
  ]);
  if (done) return;
  const endpoint = JSON.parse(endpointText);
  if (!endpoint || endpoint.transport !== 'tcp' || !Number.isInteger(endpoint.port)
      || endpoint.port < 1 || endpoint.port > 65535
      || typeof endpoint.token !== 'string' || !/^[0-9a-f]{32}$/.test(endpoint.token)) {
    return finish({ error: 'invalid_endpoint' });
  }
  privateValues.push(endpoint.token);
  const command = commandText.replace(/\r?\n$/, '');
  if (!command.trim() || /[\r\n\0]/.test(command)) {
    return finish({ error: 'command_file_must_contain_one_text_line' });
  }
  let authenticated = false, buffer = '';
  socket = net.createConnection({ host: '127.0.0.1', port: endpoint.port });
  socket.setNoDelay(true);
  socket.setEncoding('utf8');
  socket.on('connect', () => socket.write(`auth ${endpoint.token}\n`));
  socket.on('error', () => finish({ error: 'connection_error' }));
  socket.on('close', () => finish({ error: 'connection_closed' }));
  socket.on('data', chunk => {
    if (done) return;
    buffer += chunk;
    if (buffer.length > 16 * 1024 * 1024) return finish({ error: 'reply_too_large' });
    let end;
    while (!done && (end = buffer.indexOf('\n')) !== -1) {
      const line = buffer.slice(0, end);
      buffer = buffer.slice(end + 1);
      let reply;
      try { reply = JSON.parse(line); }
      catch { return finish({ error: 'invalid_json_reply' }); }
      if (!reply || typeof reply !== 'object' || Array.isArray(reply)) {
        return finish({ error: 'invalid_reply' });
      }
      if (!authenticated) {
        if (reply.ok !== true || Object.hasOwn(reply, 'error')) {
          return finish({ error: 'authentication_refused' });
        }
        authenticated = true;
        socket.write(command + '\n');
      } else {
        finish(reply, Object.hasOwn(reply, 'error') || reply.ok === false ? 1 : 0);
      }
    }
  });
}
main().catch(() => finish({ error: 'input_error' }));

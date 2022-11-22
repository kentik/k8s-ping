import { createConnection, createServer } from 'node:net';
import process from 'node:process';
import k8s from '@kubernetes/client-node';

const BYTES = process.env.BYTES     ?? 1024;
const DELAY = process.env.DELAY     ?? 1000;
const NS    = process.env.NAMESPACE ?? 'kentik';
const PORT  = process.env.PORT      ?? 1234;

let server = createServer((socket) => {
    let addr = socket.remoteAddress;
    let port = socket.remotePort;
    socket.on('data', (data) => {
        console.log(`message from ${addr}:${port}`);
        socket.write(data);
    });
});
server.listen(PORT);

let config = new k8s.KubeConfig();
config.loadFromDefault();

setInterval(async () => {
    let buffer = new Uint8Array(BYTES);

    let client = config.makeApiClient(k8s.CoreV1Api);
    let result = await client.listNamespacedPod(NS);

    for (let pod of result.body.items) {
        let { name } = pod.metadata;
        let { hostIP, podIP } = pod.status;
        let address = `${podIP}:${PORT}`;

        console.log(`ping -> ${name} @ ${address}`);

        try {
            let socket = createConnection(PORT, podIP);
            socket.on('error', console.error);
            socket.write(buffer, () => socket.end());
        } catch (e) {
            console.error(e);
        }
    }
}, DELAY);

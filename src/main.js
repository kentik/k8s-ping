import { createSocket } from 'node:dgram';
import process from 'node:process';
import k8s from '@kubernetes/client-node';

const BYTES = process.env.BYTES     ?? 1024;
const DELAY = process.env.DELAY     ?? 1000;
const NS    = process.env.NAMESPACE ?? 'kentik';
const PORT  = process.env.PORT      ?? 1234;

let server = createSocket('udp4');
server.on('message', (msg, { address, port }) => {
    console.log(`message from ${address}:${port}`);
    server.send(msg, port, address);
});
server.bind(PORT);

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

        let socket = createSocket('udp4');
        socket.send(buffer, PORT, podIP, () => socket.close());
    }
}, DELAY);

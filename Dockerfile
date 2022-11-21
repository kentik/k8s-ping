FROM node:19-alpine3.15

ENV BYTES     1024
ENV DELAY     1000
ENV NAMESPACE kentik
ENV PORT      1234

WORKDIR   /work
ADD *     /work/
RUN npm install

EXPOSE 1234
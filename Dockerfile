FROM ubuntu:22.04

ADD k8s-ping /k8s-ping

CMD ["/k8s-ping"]

EXPOSE 1234
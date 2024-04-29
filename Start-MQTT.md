# latest RabbitMQ 3.13

Start RabbitMQ in a docker dontainer and expose it's ports.

```bash
sudo docker run -it --rm --name rabbitmq -p 1883:1883 -p 15672:15672 -p 15692:15692 rabbitmq:3.13.0-management
```

docker run -it --name rabbitmq -p 1883:1883 -p 15672:15672 -p 15692:15692 rabbitmq:3.13.0-management

## Enable the mqtt plugin from another terminal.

Run the following command to apply the mqtt pluging.

```bash
sudo docker exec rabbitmq rabbitmq-plugins enable rabbitmq_mqtt
sudo docker exec rabbitmq rabbitmqctl enable_feature_flag all
```

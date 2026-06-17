docker run -it --rm \
  -v ~/synapse:/data \
  -e SYNAPSE_SERVER_NAME=my.local.server \
  -e SYNAPSE_REPORT_STATS=no \
  matrixdotorg/synapse:latest generate

docker run -d \
  --name synapse \
  -v ~/synapse:/data \
  -p 8448:8448 \
  -p 8008:8008 \
  matrixdotorg/synapse:latest

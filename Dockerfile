FROM jetpackio/devbox:latest

# Installing your devbox project
WORKDIR /code
USER root:root
RUN chown ${DEVBOX_USER}:${DEVBOX_USER} /code
USER ${DEVBOX_USER}:${DEVBOX_USER}
#COPY --chown=${DEVBOX_USER}:${DEVBOX_USER} devbox.json devbox.json
#COPY --chown=${DEVBOX_USER}:${DEVBOX_USER} devbox.lock devbox.lock
#COPY --chown=${DEVBOX_USER}:${DEVBOX_USER} blog ./blog
#COPY --chown=${DEVBOX_USER}:${DEVBOX_USER} process-compose.yml .
COPY --chown=${DEVBOX_USER}:${DEVBOX_USER} . .

RUN devbox install

EXPOSE 8080

RUN devbox run prepare
CMD ["devbox", "run", "start_server"]
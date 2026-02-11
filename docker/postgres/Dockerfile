FROM postgres:16

# Set default timezone (optional)
ENV TZ=UTC

# Copy init scripts
COPY init/*.sql /docker-entrypoint-initdb.d/

ARG BASE_IMAGE
FROM ${BASE_IMAGE}

COPY --chown=appuser:appuser content /app/content

#!/usr/bin/env bash
set -euo pipefail

# Required env vars: GCP_PROJECT_ID, GCP_REGION, GCP_REPOSITORY_ID
# Optional: GCP_SERVICE_NAME/SERVICE_NAME
: "${GCP_PROJECT_ID:?Set GCP_PROJECT_ID}"
: "${GCP_REGION:?Set GCP_REGION}"
: "${GCP_REPOSITORY_ID:?Set GCP_REPOSITORY_ID}"
SERVICE_NAME=${GCP_SERVICE_NAME:-${SERVICE_NAME:-blog}}

IMAGE="${GCP_REGION}-docker.pkg.dev/${GCP_PROJECT_ID}/${GCP_REPOSITORY_ID}/${SERVICE_NAME}:$(git rev-parse --short HEAD 2>/dev/null || echo latest)"

echo "Enabling required services (idempotent)"
gcloud services enable run.googleapis.com cloudbuild.googleapis.com artifactregistry.googleapis.com --project "$GCP_PROJECT_ID"

echo "Ensuring Artifact Registry repo $GCP_REPOSITORY_ID in $GCP_REGION"
if ! gcloud artifacts repositories describe "$GCP_REPOSITORY_ID" --location="$GCP_REGION" --project "$GCP_PROJECT_ID" >/dev/null 2>&1; then
  gcloud artifacts repositories create "$GCP_REPOSITORY_ID" \
    --repository-format=docker --location="$GCP_REGION" \
    --description="Blog images" --project "$GCP_PROJECT_ID"
fi

echo "Building and pushing with Cloud Build: $IMAGE"
gcloud builds submit --project "$GCP_PROJECT_ID" --tag "$IMAGE"

echo "Deploying to Cloud Run: $SERVICE_NAME"
gcloud run deploy "$SERVICE_NAME" \
  --image "$IMAGE" \
  --region "$GCP_REGION" \
  --allow-unauthenticated \
  --port 8080 \
  --ingress all \
  --project "$GCP_PROJECT_ID"

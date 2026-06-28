docker compose -f common-services/docker-compose.yaml down -v
docker compose -f common-services/docker-compose.yaml up -d

# Wait for garage-init container to finish writing garage.env
echo "Waiting for Garage initialization..."
docker wait garage_init

# Sync S3 variables from common-services/garage.env into .env
for var in S3_ENDPOINT S3_REGION S3_BUCKET_NAME S3_ACCESS_KEY S3_SECRET_KEY; do
  value=$(grep "^${var}=" common-services/garage.env | cut -d'=' -f2-)
  sed -i '' "s|^${var}=.*|${var}=${value}|" .env
done

sqlx migrate run --source common-services/database-migrations --database-url postgresql://plannify_user:plannify_password@localhost:5432/plannify_db

PGPASSWORD=plannify_password psql -h localhost -U plannify_user -d plannify_db -f config/test-dataset.sql
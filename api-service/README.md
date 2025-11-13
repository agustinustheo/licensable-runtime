# License API

A simple NestJS API for validating licenses stored in a PostgreSQL database.

## Prerequisites

- Node.js 18+ (or 20+ for Prisma support)
- PostgreSQL running locally or accessible
- pnpm package manager

## Setup

1. **Install dependencies**
   ```bash
   pnpm install
   ```

2. **Configure environment variables**

   Update `.env` with your database credentials:
   ```
   DB_HOST=localhost
   DB_PORT=5432
   DB_USERNAME=postgres
   DB_PASSWORD=password
   DB_NAME=license_db
   PORT=3000
   NODE_ENV=development
   ```

3. **Seed test data**
   ```bash
   pnpm seed
   ```

   This creates test licenses:
   - `valid-license-key-12345` (valid and active)
   - `expired-license-key-67890` (expired)
   - `inactive-license-key-11111` (inactive)

4. **Start the API**
   ```bash
   pnpm start:dev
   ```

## API Endpoints

### GET /license

Validates a license key against the database.

**Query Parameters:**
- `key` (required): The license key to validate

**Response:**
```json
{
  "valid": true
}
```

or

```json
{
  "valid": false
}
```

**Examples:**

Valid license:
```bash
curl "http://localhost:3000/license?key=valid-license-key-12345"
# Output: {"valid":true}
```

Invalid/Expired license:
```bash
curl "http://localhost:3000/license?key=expired-license-key-67890"
# Output: {"valid":false}
```

Missing key:
```bash
curl "http://localhost:3000/license"
# Output: {"statusCode":400,"message":"License key is required","error":"Bad Request"}
```

## Project Structure

```
src/
├── main.ts              # Application entry point
├── app.module.ts        # Main module configuration
├── controllers/
│   └── license.controller.ts  # License validation endpoint
├── services/
│   └── license.service.ts     # License validation logic
├── entities/
│   └── license.entity.ts      # License database model
└── seed.ts              # Database seeding script
```

## Database Schema

The `licenses` table includes:
- `id` (UUID, Primary Key)
- `key` (String, Unique)
- `expiresAt` (Timestamp)
- `isActive` (Boolean)
- `createdAt` (Timestamp)

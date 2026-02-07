# LittyPicky Backend

A comprehensive Rust-based backend API for a litter-picking community platform.

## Features

### ✅ Completed Infrastructure

- **Axum Web Framework** - High-performance async web server
- **PostgreSQL + PostGIS** - Geospatial database for location-based queries
- **Docker Compose** - Containerized PostgreSQL setup
- **Database Migrations** - SQLx-powered schema versioning
- **JWT Authentication** - Secure token-based auth
- **Email Service** - Lettre with Gmail SMTP support
- **Image Processing** - WebP conversion, compression, and resizing
- **Error Handling** - Comprehensive error types with proper HTTP responses
- **Logging** - Tracing-based structured logging

### 🚧 Remaining Implementation

- Authentication endpoints (register, login, email verification, password reset)
- Report CRUD operations with geospatial search
- Community verification system
- Scoring and leaderboards
- Google OAuth integration
- Admin panel
- Rate limiting
- API documentation (Swagger/OpenAPI)

## Tech Stack

- **Language**: Rust 2021
- **Framework**: Axum 0.7
- **Database**: PostgreSQL 16 with PostGIS extension
- **ORM**: SQLx (compile-time checked queries)
- **Authentication**: JWT + Argon2 password hashing
- **Email**: Lettre with Gmail SMTP
- **Image Processing**: WebP encoding with quality compression
- **Containerization**: Docker Compose

## Project Structure

```
back-end/
├── migrations/          # Database schema migrations
├── src/
│   ├── auth/           # JWT, middleware, token generation
│   ├── handlers/       # HTTP request handlers (TODO)
│   ├── models/         # Data models and DTOs
│   ├── services/       # Business logic services
│   │   ├── email_service.rs    # Email sending
│   │   ├── image_service.rs    # Image processing
│   │   ├── auth_service.rs     # (TODO)
│   │   ├── report_service.rs   # (TODO)
│   │   └── scoring_service.rs  # (TODO)
│   ├── templates/      # Email HTML templates
│   ├── config.rs       # Environment configuration
│   ├── db.rs           # Database connection pool
│   ├── error.rs        # Error types
│   └── main.rs         # Application entry point
├── Cargo.toml
├── docker-compose.yml
└── .env.example
```

## Database Schema

### Core Tables

1. **users** - User accounts with email verification and OAuth support
2. **refresh_tokens** - JWT refresh token storage
3. **email_verification_tokens** - Email verification links (24h expiry)
4. **password_reset_tokens** - Password reset links (1h expiry)
5. **litter_reports** - Geospatial litter reports with photos
6. **report_verifications** - Community verification of cleared reports
7. **user_scores** - Points, streaks, and statistics

### Key Features

- PostGIS GEOMETRY for efficient geospatial queries
- Automatic `updated_at` timestamps via triggers
- Proper foreign key relationships with CASCADE/SET NULL
- Indexes on frequently queried fields

## Environment Configuration

Copy `.env.example` to `.env` and configure:

### Required Variables

```env
# Database
DATABASE_URL=postgresql://littypicky:securepassword@localhost:5432/littypicky

# JWT
JWT_SECRET=your-super-secret-jwt-key-minimum-32-chars

# Google OAuth
GOOGLE_CLIENT_ID=your-google-client-id
GOOGLE_CLIENT_SECRET=your-google-client-secret

# Email (Gmail SMTP)
SMTP_USERNAME=your-email@gmail.com
SMTP_PASSWORD=your-app-specific-password
```

### Gmail Setup

1. Enable 2-Factor Authentication on your Gmail account
2. Generate App Password at: https://myaccount.google.com/apppasswords
3. Use the 16-character app password in `SMTP_PASSWORD`

## Getting Started

### Prerequisites

- Rust 1.70+ with Cargo
- Docker and Docker Compose
- PostgreSQL client (optional, for manual DB access)

### Installation

1. **Clone and setup**
   ```bash
   cd back-end
   cp .env.example .env
   # Edit .env with your configuration
   ```

2. **Start PostgreSQL**
   ```bash
   docker-compose up -d
   ```

3. **Run migrations**
   ```bash
   cargo sqlx migrate run
   ```

4. **Build and run**
   ```bash
   cargo run
   ```

   The server starts on `http://localhost:8080`

### Development Commands

```bash
# Check compilation
cargo check

# Run with auto-reload (install cargo-watch)
cargo watch -x run

# Run migrations
cargo sqlx migrate run

# Revert last migration
cargo sqlx migrate revert

# View logs
RUST_LOG=debug cargo run

# Stop PostgreSQL
docker-compose down
```

## API Design

### Authentication Endpoints (Planned)

```
POST   /api/auth/register              # Email/password registration
POST   /api/auth/login                 # Login with email/password
POST   /api/auth/google                # Google OAuth login
POST   /api/auth/verify-email          # Verify email with token
POST   /api/auth/resend-verification   # Resend verification email
POST   /api/auth/forgot-password       # Request password reset
POST   /api/auth/reset-password        # Reset password with token
POST   /api/auth/refresh               # Refresh access token
POST   /api/auth/logout                # Logout
```

### Report Endpoints (Planned)

```
GET    /api/reports/nearby            # Geospatial search
POST   /api/reports                   # Create report (requires verified email)
GET    /api/reports/:id               # Get report details
POST   /api/reports/:id/claim         # Claim a report
POST   /api/reports/:id/clear         # Submit cleared photo
POST   /api/reports/:id/verify        # Verify cleared report (requires 5+ clears)
```

### Leaderboard Endpoints (Planned)

```
GET    /api/leaderboards/city/:city       # Top 20 by city
GET    /api/leaderboards/country/:country # Top 20 by country
GET    /api/leaderboards/global           # Top 20 globally
```

## Image Processing

All uploaded images are automatically:

1. **Decoded** from base64
2. **Validated** for size (<5MB by default)
3. **Resized** to max 1920x1920 (preserving aspect ratio)
4. **Converted** to WebP format with 80% quality
5. **Re-encoded** to base64 for database storage

**Benefits:**
- ~70% smaller file sizes vs JPEG/PNG
- Consistent format across all images
- Controlled maximum dimensions
- Fast client-side loading

## Email Templates

Professional HTML + plain text email templates for:

- **Email Verification** - Welcome message with verification link
- **Password Reset** - Security-focused reset instructions
- **Password Reset Confirmation** - Security notification

All templates use responsive design and work across email clients.

## Scoring System (Design)

### Base Points
- 10 points per cleared report

### Multipliers
- **Streak Bonus**: +5 points per day of current streak
- **First in Area**: +20 points for first clear within 1km in 24h
- **Verification**: +2 points for verifying someone else's clear
- **Verified Report**: +10 points when your clear gets verified (3+ verifications)

### Verification Rules
- Must have 5+ cleared reports to verify others
- Cannot verify own clears or reports you submitted
- 3 positive verifications required to mark report as "verified"

## Security Features

✅ Argon2 password hashing  
✅ JWT with short expiry (15 min access, 30 day refresh)  
✅ Email verification required for email/password users  
✅ Rate limiting on sensitive endpoints  
✅ CORS configuration  
✅ SQL injection prevention (compile-time checked queries)  
✅ Secure token generation for email verification/reset  
✅ All sessions invalidated on password reset  

## Next Steps

To complete the MVP, implement:

1. **Auth Handlers** - Registration, login, email flows
2. **Report Handlers** - CRUD + geospatial search
3. **Scoring Service** - Point calculation logic
4. **Leaderboard Handlers** - Ranked queries with time filters
5. **Rate Limiting** - Using tower_governor
6. **Google OAuth** - Complete OAuth2 flow
7. **Admin Endpoints** - Moderation tools
8. **API Docs** - OpenAPI/Swagger generation

## Testing

```bash
# Run tests (when implemented)
cargo test

# Test with coverage (install cargo-tarpaulin)
cargo tarpaulin

# Lint
cargo clippy

# Format
cargo fmt
```

## Admin Setup

After deployment, manually elevate your user to admin:

```sql
UPDATE users 
SET role = 'admin' 
WHERE email = 'your-admin-email@gmail.com';
```

## Production Considerations

- [ ] Set strong JWT_SECRET (32+ random characters)
- [ ] Use dedicated email service (SendGrid, AWS SES) instead of Gmail
- [ ] Configure production CORS origins
- [ ] Set up SSL/TLS with reverse proxy (nginx, Caddy)
- [ ] Implement proper database backup strategy
- [ ] Set up monitoring and alerting
- [ ] Consider moving images to S3/object storage
- [ ] Implement rate limiting per IP/user
- [ ] Set up CI/CD pipeline
- [ ] Add comprehensive tests

## License

Proprietary - All rights reserved

## Contributing

This is a private project. Contact the maintainer for collaboration opportunities.

---

**Built with ❤️ using Rust and Axum**

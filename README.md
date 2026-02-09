# LittyPicky

LittyPicky is a community-driven platform designed to gamify litter collection and improve environmental cleanliness. Users can report litter, verify cleanups, and compete on leaderboards, fostering a social effort to keep neighborhoods clean.

See it live: <br>
[https://littypicky.nullstring.one](https://littypicky.nullstring.one)

![Home Page](docs/imgs/index.png)

## Features

- **Litter Reporting:** Users can identify and report locations of litter using an interactive map.
- **Cleanup Verification:** Community members can clean up reported litter and submit proof for verification.
- **Social Feed:** Share cleanup achievements, post updates, comment, and like other users' activities.
- **Gamification & Leaderboards:** Earn scores for reporting and cleaning up litter. Compete with others on global and local leaderboards.
- **User Profiles:** Track your contributions and history.

## Technical Overview

### Backend
Built with performance and reliability in mind using **Rust**.
- **Framework:** Axum (High-performance async web framework)
- **Database:** PostgreSQL (Data persistence for users, reports, and feeds)
- **Storage:** AWS S3 compatible object storage for image assets.
- **Authentication:** Secure user authentication using JWT, Argon2, and OAuth2/OpenID Connect.
- **Documentation:** Auto-generated OpenAPI/Swagger documentation.

### Frontend
A modern, responsive web interface built with **SvelteKit**.
- **Framework:** SvelteKit (Full-stack web application framework)
- **Styling:** Tailwind CSS (Utility-first CSS framework)
- **Maps:** Leaflet (Interactive maps for reporting and viewing litter locations)
- **Deployment:** Configured for Cloudflare (Edge deployment).

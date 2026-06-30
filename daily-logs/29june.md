# Daily Log: June 29, 2026

## What We Talked About & Solved Today
Today was highly focused on system administration, infrastructure troubleshooting, and architectural strategy!

1. **Troubleshooting Port Conflicts (`AddrInUse: 5432`)**
   - I ran into a persistent issue where `mprocs` and `make db-start` failed to boot the Docker database, throwing a `failed to bind host port 0.0.0.0:5432/tcp: address already in use` error.
   - We investigated and discovered that my Linux Mint operating system automatically starts a native, system-level PostgreSQL service (`systemd daemon`) on boot. This native service silently grabbed port `5432`, blocking Docker from using it!
   - **The Fix**: Ran `sudo systemctl stop postgresql` (and optionally `disable`) to kill the native process, freeing up the port for our containerized environment.

2. **The "Zero-Cost" Coolify Strategy**
   - I explored a brilliant workaround to deploy the final Ahlan-Commerce artifact without paying for a DigitalOcean or Hetzner VPS.
   - Since Linux Mint is Ubuntu-based under the hood, I realized I can run the Coolify installation script directly on my local metal, effectively turning my personal laptop into a PaaS server!
   - The plan was to use a free tunneling service (like Ngrok or Cloudflare Tunnels) to securely expose the local Coolify deployment port to the public internet so my mentor could access it.
   - **Blocker**: During the installation, I discovered that Coolify requires significant disk storage to operate. I have paused the local Coolify deployment until I can attach extra storage to my Linux Mint device.

## What I Learned & Studied
To strengthen my overall architectural knowledge for the Mentor Defense, I spent time studying large-scale system design concepts.

- **Core Backend Architecture**: I watched an in-depth System Design video that covered the critical pillars of scaling applications.
- **Topics Covered**:
  - APIs (REST, GraphQL)
  - Databases (SQL vs NoSQL, Sharding, Replication)
  - Caching Strategies (Redis, Memcached)
  - CDNs (Content Delivery Networks)
  - Load Balancing
  - Production Infrastructure

## Resources
- **System Design Video**: https://www.youtube.com/watch?v=C842vFY5kRo&t=1051s

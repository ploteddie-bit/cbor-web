I've been building something for 6 months that I believe will change how AI agents interact with the web. Today, it's live on 38 sites, available in 8 programming languages, and submitted as an IETF Internet-Draft. Here's what it is and why it matters.

**The problem:** Every AI agent — ChatGPT, Claude, Gemini, enterprise RAG pipelines — wastes 93% of the tokens it pays for just stripping HTML noise. Menus, ads, cookie banners, JavaScript bundles. A single agent reading 1,000 pages/day burns ~$36/day on markup alone. At millions of agents, that's billions of tokens wasted *daily*.

**The solution:** CBOR-Web is an open protocol that lets any website publish a machine-readable binary version of its content alongside its existing HTML. One command, 30 minutes, zero changes to your site. AI agents get a 100× smaller, 100% structured feed — 536 bytes instead of 1.6 MB. Same content. Radical efficiency.

**Why it matters now:**
- France is legislating digital identity for AI agents. CBOR-Web is the only protocol with native cryptographic agent identity — your site is already compliant.
- Token economics: at 100× cost reduction, web-scale AI consumption is suddenly viable. This changes the game for RAG, fine-tuning, and autonomous agents.
- It's not theoretical. 38 domains, 8 SDKs (TS, Python, Go, Rust, PHP, Ruby, Java, C++), Cloudflare edge CDN, npm package published, IETF draft submitted.

**The bet:** The web has two clients now — humans and machines. We're only serving one. CBOR-Web serves both.

I built this because I believe the infrastructure layer between AI and the web is 30 years overdue for an upgrade. If you run a content site, a docs platform, or an AI product — I'd love for you to try it.

🔗 **GitHub:** [github.com/ploteddie-bit/cbor-web](https://github.com/ploteddie-bit/cbor-web)
🔗 **Live dashboard:** [cbor-web.explodev.workers.dev](https://cbor-web.explodev.workers.dev)

#AI #WebDev #OpenSource #RAG #MachineLearning #IETF #Cloudflare

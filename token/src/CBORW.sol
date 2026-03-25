// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/token/ERC20/extensions/ERC20Burnable.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

/**
 * @title CBORW — CBOR-Web Token
 * @author Eddie Plot & Claude — Deltopide
 * @notice Utility token for the CBOR-Web protocol.
 *
 * One token, three functions:
 *   1. ACCESS  — Hold ≥1 CBORW = T1 access to premium CBOR-Web content
 *   2. PAYMENT — Spend CBORW to read premium content (burn-on-use)
 *   3. IDENTITY — Wallet address = verifiable publisher/agent identity
 *
 * Fixed supply. No mint after deployment. Burn-on-use is deflationary.
 *
 * Trust chain: index.cbor signature → DNS TXT → wallet → CBORW balance → identity
 */
contract CBORW is ERC20, ERC20Burnable, Ownable {

    // ============================================================
    // Constants
    // ============================================================

    /// @notice Total supply: 100,000,000 CBORW (fixed, never changes)
    uint256 public constant TOTAL_SUPPLY = 100_000_000;

    /// @notice Cost per content access in CBORW (0.001 CBORW = 10^15 wei)
    /// Publishers can override this per-page via index.cbor pricing
    uint256 public defaultAccessCost = 1e15; // 0.001 CBORW

    /// @notice Minimum balance for T1 access (1 CBORW)
    uint256 public constant T1_MIN_BALANCE = 1e18; // 1 CBORW

    // ============================================================
    // Events
    // ============================================================

    /// @notice Emitted when an agent accesses premium content
    event ContentAccess(
        address indexed agent,
        address indexed publisher,
        string domain,
        uint256 cost,
        uint256 timestamp
    );

    /// @notice Emitted when a publisher registers their domain
    event PublisherRegistered(
        address indexed wallet,
        string domain,
        uint256 timestamp
    );

    // ============================================================
    // State
    // ============================================================

    /// @notice Publisher wallet → domain mapping
    mapping(address => string) public publisherDomain;

    /// @notice Domain → publisher wallet (reverse lookup)
    mapping(string => address) public domainPublisher;

    /// @notice Publisher custom pricing per domain (0 = use default)
    mapping(string => uint256) public domainAccessCost;

    // ============================================================
    // Constructor — mint everything, distribute later
    // ============================================================

    constructor() ERC20("CBOR-Web Token", "CBORW") Ownable(msg.sender) {
        // Mint entire supply to deployer (Deltopide)
        // Distribution happens via transfers after deployment:
        //   20% Founder (Deltopide)  = 20,000,000 CBORW
        //   40% Verifiers/Rewards    = 40,000,000 CBORW
        //   20% Community/Grants     = 20,000,000 CBORW
        //   10% Development          = 10,000,000 CBORW
        //   10% Strategic Reserve    = 10,000,000 CBORW
        _mint(msg.sender, TOTAL_SUPPLY * 10 ** decimals());
    }

    // ============================================================
    // Publisher Registration
    // ============================================================

    /// @notice Register a domain to your wallet (publisher identity)
    /// @param domain The domain name (e.g., "deltopide.fr")
    function registerDomain(string calldata domain) external {
        require(bytes(domain).length > 0, "Empty domain");
        require(
            domainPublisher[domain] == address(0) || domainPublisher[domain] == msg.sender,
            "Domain already registered by another wallet"
        );

        // Clear old domain if switching
        string memory oldDomain = publisherDomain[msg.sender];
        if (bytes(oldDomain).length > 0) {
            delete domainPublisher[oldDomain];
        }

        publisherDomain[msg.sender] = domain;
        domainPublisher[domain] = msg.sender;

        emit PublisherRegistered(msg.sender, domain, block.timestamp);
    }

    /// @notice Set custom access cost for your domain
    /// @param cost Cost in CBORW wei (e.g., 1e15 = 0.001 CBORW)
    function setAccessCost(uint256 cost) external {
        string memory domain = publisherDomain[msg.sender];
        require(bytes(domain).length > 0, "Register domain first");
        domainAccessCost[domain] = cost;
    }

    // ============================================================
    // Content Access (burn-on-use)
    // ============================================================

    /// @notice Access premium content — burns tokens as payment
    /// @param domain The publisher's domain
    /// @dev Agent calls this to unlock T1 content. Tokens are burned, not transferred.
    function accessContent(string calldata domain) external {
        require(balanceOf(msg.sender) >= T1_MIN_BALANCE, "Insufficient CBORW for T1 access");

        address publisher = domainPublisher[domain];
        uint256 cost = domainAccessCost[domain];
        if (cost == 0) cost = defaultAccessCost;

        require(balanceOf(msg.sender) >= cost, "Insufficient CBORW for access cost");

        // Burn the access cost (deflationary)
        _burn(msg.sender, cost);

        emit ContentAccess(msg.sender, publisher, domain, cost, block.timestamp);
    }

    // ============================================================
    // View Functions
    // ============================================================

    /// @notice Check if an address has T1 access (holds ≥1 CBORW)
    function hasT1Access(address account) external view returns (bool) {
        return balanceOf(account) >= T1_MIN_BALANCE;
    }

    /// @notice Get the access cost for a domain
    function getAccessCost(string calldata domain) external view returns (uint256) {
        uint256 cost = domainAccessCost[domain];
        return cost == 0 ? defaultAccessCost : cost;
    }

    /// @notice Get publisher info for a domain
    function getPublisher(string calldata domain) external view returns (address wallet, uint256 balance) {
        wallet = domainPublisher[domain];
        balance = wallet != address(0) ? balanceOf(wallet) : 0;
    }

    // ============================================================
    // Admin (owner only — Deltopide)
    // ============================================================

    /// @notice Update default access cost (governance)
    function setDefaultAccessCost(uint256 cost) external onlyOwner {
        defaultAccessCost = cost;
    }
}

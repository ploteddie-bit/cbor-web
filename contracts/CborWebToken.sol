// SPDX-License-Identifier: MIT
// CBOR-Web Token (CBORW) — ERC-20 Utility Token
// Hold-to-access model: holding ≥ 1 CBORW grants full access to all CBOR-Web L1 content
// Compatible with OpenZeppelin Contracts v5.x
pragma solidity ^0.8.20;

import {ERC20} from "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";
import {ERC20Permit} from "@openzeppelin/contracts/token/ERC20/extensions/ERC20Permit.sol";

/// @title CBOR-Web Token
/// @notice ERC-20 utility token for CBOR-Web protocol access
/// @dev Hold ≥ MIN_HOLD tokens to access L1 content on any CBOR-Web site
contract CborWebToken is ERC20, Ownable, ERC20Permit {

    // ── Constants ──

    uint256 public constant TOTAL_SUPPLY = 100_000_000 * 1 ether; // 100M tokens (18 decimals)
    uint256 public constant MIN_HOLD = 1 ether;                  // 1 token minimum for access
    uint256 public constant FOUNDER_ALLOCATION = 18_000_000 * 1 ether; // 18M (18%)
    uint256 public constant ECOSYSTEM_ALLOCATION = 40_000_000 * 1 ether; // 40M (40%)
    uint256 public constant COMMUNITY_ALLOCATION = 20_000_000 * 1 ether; // 20M (20%)
    uint256 public constant DEV_ALLOCATION = 10_000_000 * 1 ether;       // 10M (10%)
    uint256 public constant LIQUIDITY_ALLOCATION = 8_000_000 * 1 ether;  // 8M (8%)
    uint256 public constant ADVISOR_ALLOCATION = 4_000_000 * 1 ether;    // 4M (4%)

    // ── State ──

    address public founder;
    address public ecosystemVault;
    address public communityVault;
    address public devVault;
    address public liquidityVault;
    address public advisorVault;

    bool public founderLocked = true;
    uint256 public founderUnlockTime;
    uint256 public constant FOUNDER_LOCK_DURATION = 365 days;

    // ── Events ──

    event AccessVerified(address indexed holder, bool hasAccess, uint256 balance);
    event FounderUnlocked();

    constructor() ERC20("CBOR-Web Token", "CBORW") Ownable(msg.sender) ERC20Permit("CBOR-Web Token") {
        founder = msg.sender;
        founderUnlockTime = block.timestamp + FOUNDER_LOCK_DURATION;

        _mint(founder, FOUNDER_ALLOCATION);
        _mint(address(this), ECOSYSTEM_ALLOCATION);
        _mint(address(this), COMMUNITY_ALLOCATION);
        _mint(address(this), DEV_ALLOCATION);
        _mint(address(this), LIQUIDITY_ALLOCATION);
        _mint(address(this), ADVISOR_ALLOCATION);
    }

    // ── CBOR-Web Access Verification ──

    /// @notice Check if an address has CBOR-Web access (holds ≥ MIN_HOLD tokens)
    /// @param holder The address to check
    /// @return hasAccess True if the holder has at least MIN_HOLD tokens
    function verifyAccess(address holder) external view returns (bool hasAccess, uint256 balance) {
        balance = balanceOf(holder);
        hasAccess = balance >= MIN_HOLD;
        return (hasAccess, balance);
    }

    /// @notice Batch check access for multiple addresses
    function verifyAccessBatch(address[] calldata holders) external view returns (bool[] memory, uint256[] memory) {
        uint256 len = holders.length;
        bool[] memory access = new bool[](len);
        uint256[] memory balances = new uint256[](len);
        for (uint256 i = 0; i < len; i++) {
            balances[i] = balanceOf(holders[i]);
            access[i] = balances[i] >= MIN_HOLD;
        }
        return (access, balances);
    }

    // ── Admin ──

    /// @notice Allocate from contract-held reserves to a vault address
    function allocate(address to, uint256 amount) external onlyOwner {
        require(balanceOf(address(this)) >= amount, "Insufficient contract balance");
        _transfer(address(this), to, amount);
    }

    /// @notice Unlock founder allocation after 12-month cliff
    function unlockFounder() external onlyOwner {
        require(founderLocked, "Already unlocked");
        require(block.timestamp >= founderUnlockTime, "Still locked");
        founderLocked = false;
        emit FounderUnlocked();
    }

    // ── Overrides ──

    function _update(address from, address to, uint256 value) internal override(ERC20) {
        super._update(from, to, value);
    }
}

// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Script.sol";
import "../src/CBORW.sol";

contract DeployScript is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");

        vm.startBroadcast(deployerPrivateKey);

        CBORW token = new CBORW();

        console.log("CBORW deployed at:", address(token));
        console.log("Total supply:", token.totalSupply());
        console.log("Owner:", token.owner());

        vm.stopBroadcast();
    }
}

import BigInt
import Foundation
import Primitives

public protocol ChainServiceable: Sendable {
    func getChainID() async throws -> String
    func getLatestBlock() async throws -> BigInt
    func getNodeStatus(url: String) async throws -> NodeStatus

    func getValidators(apr: Double) async throws -> [DelegationValidator]
    func getDelegationValidators(address: String) async throws -> [DelegationValidator]
    func getStakeDelegations(address: String) async throws -> [DelegationBase]

    func getTokenData(tokenId: String) async throws -> Asset
    func getIsTokenAddress(tokenId: String) async throws -> Bool
}

public extension ChainServiceable {

    func getValidators(apr _: Double) async throws -> [DelegationValidator] {
        []
    }

    func getDelegationValidators(address _: String) async throws -> [DelegationValidator] {
        []
    }

    func getStakeDelegations(address _: String) async throws -> [DelegationBase] {
        []
    }

    func getTokenData(tokenId _: String) async throws -> Asset {
        throw AnyError("Not Implemented")
    }

    func getIsTokenAddress(tokenId _: String) -> Bool {
        false
    }
}

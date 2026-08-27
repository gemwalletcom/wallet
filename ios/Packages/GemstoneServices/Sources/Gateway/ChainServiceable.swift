import BigInt
import Foundation
import Primitives

public protocol ChainServiceable: Sendable {
    func defaultPriority(for type: TransferDataType) -> FeePriority

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
    func defaultPriority(for type: TransferDataType) -> FeePriority {
        switch type {
        case let .swap(fromAsset, _, _): fromAsset.chain == .bitcoin ? .fast : .normal
        case .tokenApprove, .stake, .transfer, .deposit, .transferNft, .generic, .account, .perpetual, .withdrawal, .earn: .normal
        }
    }

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

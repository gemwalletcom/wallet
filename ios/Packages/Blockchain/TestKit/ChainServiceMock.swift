// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Blockchain
import Foundation
import Primitives

public final class ChainServiceMock: ChainServiceable, @unchecked Sendable {
    // Injected data
    public var addressStatuses: [AddressStatus] = []
    public var coinBalances: [String: AssetBalance] = [:]
    public var tokenBalances: [String: [AssetBalance]] = [:]
    public var stakeBalance: AssetBalance?
    public var broadcastResponses: [String] = []
    public var fee: Fee = .init(fee: .zero, gasPriceType: .regular(gasPrice: .zero), gasLimit: .zero)
    public var feeRates: [FeeRate] = []
    public var chainID: String?
    public var latestBlock: BigInt = .zero
    public var validators: [DelegationValidator] = []
    public var delegations: [DelegationBase] = []
    public var inSync: Bool = true
    public var tokenData: [String: Asset] = [:]
    public var transactionData: TransactionData = .init(fee: Fee(fee: .zero, gasPriceType: .regular(gasPrice: .zero), gasLimit: .zero))
    public var transactionPreload: TransactionLoadMetadata = .none
    public var transactionState: TransactionChanges = .init(state: .pending, changes: [])
    public var nodeStatus: NodeStatus = .init(chainId: "1", latestBlockNumber: .zero, latency: .from(duration: 1000))

    public init() {}
}

public extension ChainServiceMock {
    func getAddressStatus(address _: String) async throws -> [AddressStatus] {
        addressStatuses
    }

    func coinBalance(for address: String) async throws -> AssetBalance {
        coinBalances[address] ?? .init(assetId: AssetId(chain: .ethereum, tokenId: nil), balance: .zero)
    }

    func tokenBalance(for address: String, tokenIds: [AssetId]) async throws -> [AssetBalance] {
        tokenBalances[address] ?? tokenIds.map { AssetBalance(assetId: $0, balance: .zero) }
    }

    func getStakeBalance(for _: String) async throws -> AssetBalance? {
        stakeBalance
    }

    func getEarnBalance(for _: String, tokenIds _: [AssetId]) async throws -> [AssetBalance] {
        []
    }

    func broadcast(data _: String, options _: BroadcastOptions) async throws -> String {
        broadcastResponses.removeFirst()
    }

    func getFee(asset _: Asset, input _: FeeInput) async throws -> Fee {
        fee
    }

    func feeRates(type _: TransferDataType) async throws -> [FeeRate] {
        feeRates
    }

    func getChainID() async throws -> String {
        chainID ?? ""
    }

    func getLatestBlock() async throws -> BigInt {
        latestBlock
    }

    func getValidators(apr _: Double) async throws -> [DelegationValidator] {
        validators
    }

    func getStakeDelegations(address _: String) async throws -> [DelegationBase] {
        delegations
    }

    func getInSync() async throws -> Bool {
        inSync
    }

    func getTokenData(tokenId: String) async throws -> Asset {
        tokenData[tokenId] ?? Asset(
            id: AssetId(chain: .ethereum, tokenId: nil),
            name: "Ethereum",
            symbol: "ETH",
            decimals: 18,
            type: .native,
        )
    }

    func getIsTokenAddress(tokenId: String) -> Bool {
        tokenData[tokenId] != nil
    }

    func load(input _: TransactionInput) async throws -> TransactionData {
        transactionData
    }

    func preload(input _: TransactionPreloadInput) async throws -> TransactionLoadMetadata {
        transactionPreload
    }

    func transactionState(for _: TransactionStateRequest) async throws -> TransactionChanges {
        transactionState
    }

    func getNodeStatus(url _: String) async throws -> NodeStatus {
        nodeStatus
    }
}

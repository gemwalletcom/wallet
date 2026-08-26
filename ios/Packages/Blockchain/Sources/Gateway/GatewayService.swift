// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import Gemstone
import NativeProviderService
import Primitives

public import GemstonePrimitives

public actor GatewayService: Sendable {
    let gateway: GemGateway

    public init(
        provider: NativeProvider,
    ) {
        gateway = GemGateway(
            provider: provider,
            preferences: GemstonePreferences(namespace: "gateway"),
            securePreferences: GemstoneSecurePreferences(namespace: "gateway"),
        )
    }

    public nonisolated func stakeService(staticApi: GemStaticApiClient, store: any GemStakeStore) -> GemStakeService {
        GemStakeService(gateway: gateway, staticApi: staticApi, store: store)
    }

    public nonisolated func confirmService(
        simulation: TransactionSimulationService,
        scanner: GemScanService,
    ) -> GemConfirmService {
        GemConfirmService(gateway: gateway, simulation: simulation, scanner: scanner)
    }
}

// MARK: - Balances

public extension GatewayService {
    func coinBalance(chain: Primitives.Chain, address: String) async throws -> AssetBalance {
        try await gateway.getBalanceCoin(chain: chain.rawValue, address: address).map()
    }

    func tokenBalance(chain: Primitives.Chain, address: String, tokenIds: [Primitives.AssetId]) async throws -> [AssetBalance] {
        try await gateway
            .getBalanceTokens(chain: chain.rawValue, address: address, tokenIds: tokenIds.compactMap(\.tokenId))
            .map { try $0.map() }
    }

    func getStakeBalance(chain: Primitives.Chain, address: String) async throws -> AssetBalance? {
        try await gateway.getBalanceStaking(chain: chain.rawValue, address: address)?.map()
    }

    func getEarnBalance(chain: Primitives.Chain, address: String, tokenIds: [Primitives.AssetId]) async throws -> [AssetBalance] {
        try await gateway.getBalanceEarn(chain: chain.rawValue, address: address, tokenIds: tokenIds.compactMap(\.tokenId))
            .map { try $0.map() }
    }
}

// MARK: - Transactions

public extension GatewayService {
    func transactionUpdate(_ transaction: Primitives.Transaction) async throws -> TransactionChanges {
        try await gateway.getTransactionUpdate(transaction: transaction.json()).map()
    }
}

// MARK: - Account

extension GatewayService {
    func utxos(chain: Primitives.Chain, address: String) async throws -> [Primitives.UTXO] {
        try await gateway.getUtxos(chain: chain.rawValue, address: address).map {
            try Primitives.UTXO($0)
        }
    }
}

// TransactionPreload

// MARK: - State

public extension GatewayService {
    func chainId(chain: Primitives.Chain) async throws -> String {
        try await gateway.getChainId(chain: chain.rawValue)
    }

    func latestBlock(chain: Primitives.Chain) async throws -> BigInt {
        try await gateway.getBlockNumber(chain: chain.rawValue).asBigInt
    }

    func feeRates(chain: Primitives.Chain, input: TransferDataType) async throws -> [FeeRate] {
        try await gateway.getFeeRates(chain: chain.rawValue, input: input.map()).map { try $0.map() }
    }

    func nodeStatus(chain: Primitives.Chain, url: String) async throws -> Primitives.NodeStatus {
        try await gateway.getNodeStatus(chain: chain.rawValue, url: url).map()
    }
}

// MARK: - Token

public extension GatewayService {
    func tokenData(chain: Primitives.Chain, tokenId: String) async throws -> Primitives.Asset {
        try await Primitives.Asset(gateway.getTokenData(chain: chain.rawValue, tokenId: tokenId))
    }

    func isTokenAddress(chain: Primitives.Chain, tokenId: String) async throws -> Bool {
        try await gateway.getIsTokenAddress(chain: chain.rawValue, tokenId: tokenId)
    }
}

// MARK: - Transaction Preload

public extension GatewayService {
    func transactionPreload(chain: Primitives.Chain, input: TransactionPreloadInput) async throws -> GemTransactionLoadMetadata {
        try await gateway.getTransactionPreload(chain: chain.rawValue, input: input.map())
    }

    func transactionLoad(chain: Primitives.Chain, input: GemTransactionLoadInput) async throws -> TransactionData {
        try await gateway.getTransactionLoad(chain: chain.rawValue, input: input).map()
    }
}

// MARK: - Staking

public extension GatewayService {
    func validators(chain: Primitives.Chain, apy: Double) async throws -> [Primitives.DelegationValidator] {
        try await gateway.getStakingValidators(chain: chain.rawValue, apy: apy)
            .map { try Primitives.DelegationValidator($0) }
    }

    func delegationValidators(chain: Primitives.Chain, address: String) async throws -> [Primitives.DelegationValidator] {
        try await gateway.getStakingDelegationValidators(chain: chain.rawValue, address: address)
            .map { try Primitives.DelegationValidator($0) }
    }

    func delegations(chain: Primitives.Chain, address: String) async throws -> [Primitives.DelegationBase] {
        try await gateway.getStakingDelegations(chain: chain.rawValue, address: address)
            .map { try Primitives.DelegationBase($0) }
    }
}

// MARK: - Earn

public extension GatewayService {
    func earnProviders(assetId: Primitives.AssetId) throws -> [Primitives.DelegationValidator] {
        try gateway.getEarnProviders(assetId: assetId.identifier).map { try Primitives.DelegationValidator($0) }
    }

    func earnPositions(address: String, assetId: Primitives.AssetId) async throws -> [Primitives.DelegationBase] {
        try await gateway.getEarnPositions(address: address, assetId: assetId.identifier).map { try Primitives.DelegationBase($0) }
    }

    func getEarnData(
        assetId: Primitives.AssetId,
        address: String,
        value: String,
        earnType: Primitives.EarnType,
    ) async throws -> Primitives.ContractCallData {
        try await Primitives.ContractCallData(
            gateway.getEarnData(assetId: assetId.identifier, address: address, value: value, earnType: earnType.json()),
        )
    }
}

// MARK: - Perpetual

public extension GatewayService {
    func getPositions(chain: Primitives.Chain, address: String) async throws -> Primitives.PerpetualPositionsSummary {
        try await Primitives.PerpetualPositionsSummary(gateway.getPositions(chain: chain.rawValue, address: address))
    }

    func getPerpetualAccountMode(chain: Primitives.Chain, address: String) async throws -> Primitives.PerpetualAccountMode {
        try await Primitives.PerpetualAccountMode(gateway.getPerpetualAccountMode(chain: chain.rawValue, address: address))
    }

    func getPerpetualsData(chain: Primitives.Chain) async throws -> [Primitives.PerpetualData] {
        try await gateway.getPerpetualsData(chain: chain.rawValue).map {
            try Primitives.PerpetualData($0)
        }
    }

    func getPerpetualCandlesticks(chain: Primitives.Chain, symbol: String, period: Primitives.ChartPeriod) async throws -> [Primitives.ChartCandleStick] {
        try await gateway.getPerpetualCandlesticks(chain: chain.rawValue, symbol: symbol, period: period.rawValue).map {
            try Primitives.ChartCandleStick($0)
        }
    }

    func getPerpetualPortfolio(chain: Primitives.Chain, address: String) async throws -> Primitives.PerpetualPortfolio {
        try await Primitives.PerpetualPortfolio(gateway.getPerpetualPortfolio(chain: chain.rawValue, address: address))
    }
}

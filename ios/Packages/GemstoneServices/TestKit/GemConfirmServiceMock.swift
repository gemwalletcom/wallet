// Copyright (c). Gem Wallet. All rights reserved.

public import typealias Gemstone.AssetId
public import typealias Gemstone.Chain
public import struct Gemstone.GemAssetBalance
public import struct Gemstone.GemConfirmData
public import enum Gemstone.GemConfirmError
public import struct Gemstone.GemConfirmInput
public import struct Gemstone.GemConfirmLoadOptions
public import struct Gemstone.GemConfirmMetadata
public import struct Gemstone.GemConfirmPreload
public import protocol Gemstone.GemConfirmServiceProtocol
public import struct Gemstone.GemConfirmSimulation
public import enum Gemstone.GemExecuteResult
public import struct Gemstone.GemFeeAsset
public import enum Gemstone.GemTransactionInputType
public import protocol Gemstone.GemTransactionSigner
public import typealias Gemstone.SimulationResult
public import typealias Gemstone.Transaction
public import typealias Gemstone.WalletId
import Foundation
import GemstonePrimitives
import Primitives

public final class GemConfirmServiceMock: GemConfirmServiceProtocol, @unchecked Sendable {
    private let executeResult: Result<GemExecuteResult, any Error>
    private let metadataResult: Result<GemConfirmMetadata, any Error>
    private let feeAssetRows: [GemFeeAsset]
    private let preloadResult: Result<GemConfirmPreload, any Error>
    public let simulation: GemConfirmSimulation?
    private let lock = NSLock()
    private var inputs: [GemConfirmData] = []

    public var executedInputs: [GemConfirmData] { lock.withLock { inputs } }

    public init(
        execute: Result<GemExecuteResult, any Error> = .success(.sent(hashes: [], transactions: [])),
        metadata: Result<GemConfirmMetadata, any Error>? = nil,
        feeAssets: [GemFeeAsset] = [],
        preload: Result<GemConfirmPreload, any Error> = .failure(GemConfirmError.FeeRatesMissing),
        simulation: GemConfirmSimulation? = nil,
    ) {
        executeResult = execute
        metadataResult = metadata ?? .success(GemConfirmMetadata(
            assetBalance: GemAssetBalance(assetId: Primitives.Asset.mock().id.identifier, available: "0", frozen: "0", locked: "0", staked: "0", pending: "0", pendingUnconfirmed: "0", rewards: "0", reserved: "0", withdrawable: "0", earn: "0", metadata: nil),
            feeAssetBalance: GemAssetBalance(assetId: Primitives.Asset.mock().id.identifier, available: "0", frozen: "0", locked: "0", staked: "0", pending: "0", pendingUnconfirmed: "0", rewards: "0", reserved: "0", withdrawable: "0", earn: "0", metadata: nil),
            prices: [],
        ))
        feeAssetRows = feeAssets
        preloadResult = preload
        self.simulation = simulation
    }

    public func load(input _: GemConfirmInput, options _: GemConfirmLoadOptions) async throws -> GemConfirmData {
        fatalError("not used")
    }

    public func metadata(walletId _: WalletId, assetId _: AssetId, feeAssetId _: AssetId, extraAssetIds _: [AssetId]) throws -> GemConfirmMetadata {
        try metadataResult.get()
    }

    public func preload(walletId _: WalletId, input _: GemConfirmInput, options _: GemConfirmLoadOptions) async throws -> GemConfirmPreload {
        try preloadResult.get()
    }

    public func feeAssets(walletId _: WalletId, chain _: Chain) async throws -> [GemFeeAsset] {
        feeAssetRows
    }

    public func syncMissingAssets(assetIds: [AssetId]) async throws -> [AssetId] {
        assetIds
    }

    public func trackPending() async throws {}

    public func track(walletId _: WalletId, transactions _: [Transaction]) async throws {}

    public func execute(confirm: GemConfirmData, signer _: any GemTransactionSigner) async throws -> GemExecuteResult {
        lock.withLock { inputs.append(confirm) }
        return try executeResult.get()
    }
}

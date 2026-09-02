// Copyright (c). Gem Wallet. All rights reserved.

public import struct Gemstone.GemConfirmData
public import struct Gemstone.GemConfirmInput
public import struct Gemstone.GemConfirmLoadOptions
public import struct Gemstone.GemConfirmMetadata
public import struct Gemstone.GemAssetBalance
public import enum Gemstone.GemConfirmError
public import typealias Gemstone.AssetId
public import struct Gemstone.GemFeeAsset
public import struct Gemstone.GemConfirmPreload
public import struct Gemstone.GemConfirmSimulation
public import class Gemstone.GemSimulationFormatter
public import enum Gemstone.GemTransactionInputType
public import typealias Gemstone.SimulationResult
public import typealias Gemstone.Chain
public import typealias Gemstone.WalletId
public import protocol Gemstone.GemConfirmServiceProtocol
public import enum Gemstone.GemExecuteResult
public import struct Gemstone.GemSendInput
public import typealias Gemstone.Transaction
public import protocol Gemstone.GemTransactionSigner
import Foundation
import GemstonePrimitives
import Primitives

public final class GemConfirmServiceMock: GemConfirmServiceProtocol, @unchecked Sendable {
    private let executeResult: Result<GemExecuteResult, any Error>
    private let metadataResult: Result<GemConfirmMetadata, any Error>
    private let feeAssetRows: [GemFeeAsset]
    private let preloadResult: Result<GemConfirmPreload, any Error>
    private let simulationResult: GemConfirmSimulation?
    private let simulationFormatter = GemSimulationFormatter()
    private let lock = NSLock()
    private var inputs: [GemSendInput] = []

    public var executedInputs: [GemSendInput] { lock.withLock { inputs } }

    public init(
        execute: Result<GemExecuteResult, any Error> = .success(.sent(hashes: [], transactions: [])),
        metadata: Result<GemConfirmMetadata, any Error>? = nil,
        feeAssets: [GemFeeAsset] = [],
        preload: Result<GemConfirmPreload, any Error> = .failure(GemConfirmError.FeeRatesMissing),
        simulation: GemConfirmSimulation? = nil,
    ) {
        executeResult = execute
        metadataResult = metadata ?? .success(GemConfirmMetadata(
            assetBalance: GemAssetBalance(assetId: "", available: "0", frozen: "0", locked: "0", staked: "0", pending: "0", pendingUnconfirmed: "0", rewards: "0", reserved: "0", withdrawable: "0", earn: "0", metadata: nil),
            feeAssetBalance: GemAssetBalance(assetId: "", available: "0", frozen: "0", locked: "0", staked: "0", pending: "0", pendingUnconfirmed: "0", rewards: "0", reserved: "0", withdrawable: "0", earn: "0", metadata: nil),
            prices: [],
        ))
        feeAssetRows = feeAssets
        preloadResult = preload
        simulationResult = simulation
    }

    public func load(input _: GemConfirmInput, options _: GemConfirmLoadOptions) async throws -> GemConfirmData {
        fatalError("not used")
    }

    public func metadata(walletId _: WalletId, assetId _: AssetId, feeAssetId _: AssetId, extraAssetIds _: [AssetId]) throws -> GemConfirmMetadata {
        try metadataResult.get()
    }

    public func simulation(inputType: GemTransactionInputType, simulation: SimulationResult?) throws -> GemConfirmSimulation {
        if let simulationResult {
            return simulationResult
        }
        let isApproval = if case .tokenApprove = inputType { true } else { false }
        let showsHeader = simulationFormatter.showsHeader(simulation: simulation, isApproval: isApproval)
        let payload = simulation.flatMap { try? Primitives.SimulationResult($0) }?.payload ?? []
        let fields = simulationFormatter.payloadFields(payload: payload.map { $0.json() }, showsHeader: showsHeader)
            .compactMap { try? Primitives.SimulationPayloadField($0) }
        return GemConfirmSimulation(
            primaryFields: fields.filter { $0.display == .primary }.map { $0.json() },
            secondaryFields: fields.filter { $0.display == .secondary }.map { $0.json() },
            header: nil,
            balanceChanges: [],
        )
    }

    public func preload(walletId _: WalletId, input _: GemConfirmInput, options _: GemConfirmLoadOptions) async throws -> GemConfirmPreload {
        try preloadResult.get()
    }

    public func feeAssets(walletId _: WalletId, chain _: Chain) throws -> [GemFeeAsset] {
        feeAssetRows
    }

    public func syncMissingAssets(assetIds: [AssetId]) async throws -> [AssetId] {
        assetIds
    }

    public func trackPending() async throws {}

    public func track(walletId _: WalletId, transactions _: [Transaction]) async throws {}

    public func execute(input: GemSendInput, signer _: any GemTransactionSigner) async throws -> GemExecuteResult {
        lock.withLock { inputs.append(input) }
        return try executeResult.get()
    }
}

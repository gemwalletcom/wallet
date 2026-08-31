// Copyright (c). Gem Wallet. All rights reserved.

public import struct Gemstone.GemConfirmData
public import struct Gemstone.GemConfirmInput
public import struct Gemstone.GemConfirmLoadOptions
public import struct Gemstone.GemConfirmMetadata
public import enum Gemstone.GemConfirmError
public import typealias Gemstone.AssetId
public import struct Gemstone.GemFeeAsset
public import struct Gemstone.GemConfirmPreload
public import typealias Gemstone.Chain
public import typealias Gemstone.WalletId
public import protocol Gemstone.GemConfirmServiceProtocol
public import enum Gemstone.GemExecuteResult
public import struct Gemstone.GemSendInput
public import protocol Gemstone.GemTransactionSigner
import Foundation

public final class GemConfirmServiceMock: GemConfirmServiceProtocol, @unchecked Sendable {
    private let executeResult: Result<GemExecuteResult, any Error>
    private let metadataResult: Result<GemConfirmMetadata, any Error>
    private let feeAssetRows: [GemFeeAsset]
    private let preloadResult: Result<GemConfirmPreload, any Error>
    private let lock = NSLock()
    private var inputs: [GemSendInput] = []

    public var executedInputs: [GemSendInput] { lock.withLock { inputs } }

    public init(
        execute: Result<GemExecuteResult, any Error> = .success(.sent(hashes: [], transactions: [])),
        metadata: Result<GemConfirmMetadata, any Error> = .failure(GemConfirmError.BalanceMissing(assetId: "")),
        feeAssets: [GemFeeAsset] = [],
        preload: Result<GemConfirmPreload, any Error> = .failure(GemConfirmError.FeeRatesMissing),
    ) {
        executeResult = execute
        metadataResult = metadata
        feeAssetRows = feeAssets
        preloadResult = preload
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

    public func feeAssets(walletId _: WalletId, chain _: Chain) throws -> [GemFeeAsset] {
        feeAssetRows
    }

    public func execute(input: GemSendInput, signer _: any GemTransactionSigner) async throws -> GemExecuteResult {
        lock.withLock { inputs.append(input) }
        return try executeResult.get()
    }
}

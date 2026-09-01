// Copyright (c). Gem Wallet. All rights reserved.

public import protocol Gemstone.GemConfirmTransferServiceProtocol
public import protocol Gemstone.GemConfirmServiceProtocol
public import protocol Gemstone.GemNameServiceProtocol
public import protocol Gemstone.GemAssetsServiceProtocol
public import protocol Gemstone.GemTransactionStateServiceProtocol
public import protocol Gemstone.GemTransactionSigner
public import struct Gemstone.GemBlockExplorerLink
public import struct Gemstone.GemConfirmInput
public import struct Gemstone.GemConfirmLoadOptions
public import struct Gemstone.GemConfirmMetadata
public import struct Gemstone.GemConfirmPreload
public import struct Gemstone.GemConfirmSimulation
public import struct Gemstone.GemConfirmSceneState
public import struct Gemstone.GemFeeAsset
public import struct Gemstone.GemRecentActivity
public import struct Gemstone.GemSendInput
public import struct Gemstone.GemAutocloseSummary
public import enum Gemstone.GemExecuteResult
public import enum Gemstone.GemTransactionInputType
public import enum Gemstone.GemAcquireAssetFlow
public import class Gemstone.GemApplicationMetadataService
public import class Gemstone.GemAssetConfigService
public import class Gemstone.GemFeeService
public import class Gemstone.GemSwapQuoteService
public import class Gemstone.GemTransferService
public import typealias Gemstone.AddressName
public import typealias Gemstone.AssetId
public import typealias Gemstone.Chain
public import typealias Gemstone.ChainAddress
public import typealias Gemstone.FeePriority
public import typealias Gemstone.PerpetualModifyConfirmData
public import typealias Gemstone.SimulationResult
public import typealias Gemstone.Transaction
public import typealias Gemstone.WalletId
public import typealias Gemstone.Asset
public import typealias Gemstone.TransactionType
public import typealias Gemstone.TransferDataOutputAction
import Foundation
import GemstonePrimitives
import GemstonePrimitivesTestKit

public final class GemConfirmTransferServiceMock: GemConfirmTransferServiceProtocol, @unchecked Sendable {
    private let confirm: any GemConfirmServiceProtocol
    private let names: any GemNameServiceProtocol
    private let assets: any GemAssetsServiceProtocol
    private let transactionState: any GemTransactionStateServiceProtocol
    private let feeService = GemFeeService()
    private let transferService = GemTransferService()
    private let assetConfig = GemAssetConfigService()
    private let swapQuoteService = GemSwapQuoteService()
    private let applicationMetadataService = GemApplicationMetadataService()

    public init(
        confirm: any GemConfirmServiceProtocol = GemConfirmServiceMock(),
        names: any GemNameServiceProtocol = GemNameServiceMock(),
        assets: any GemAssetsServiceProtocol = GemAssetsServiceMock(),
        transactionState: any GemTransactionStateServiceProtocol = GemTransactionStateServiceMock(),
    ) {
        self.confirm = confirm
        self.names = names
        self.assets = assets
        self.transactionState = transactionState
    }

    public func metadata(walletId: WalletId, inputType: GemTransactionInputType) throws -> GemConfirmMetadata {
        try confirm.metadata(
            walletId: walletId,
            assetId: transferService.asset(inputType: inputType).id,
            feeAssetId: transferService.feeAsset(inputType: inputType).id,
            extraAssetIds: transferService.assetIds(inputType: inputType),
        )
    }

    public func sceneState(walletId: WalletId, inputType: GemTransactionInputType, simulation result: SimulationResult?) -> GemConfirmSceneState {
        GemConfirmSceneState(
            feePriority: feeService.defaultPriority(inputType: inputType),
            feeAsset: transferService.feeAsset(inputType: inputType),
            metadata: try? metadata(walletId: walletId, inputType: inputType),
            simulation: try? simulation(inputType: inputType, simulation: result),
        )
    }

    public func feeAssets(walletId: WalletId, chain: Chain) throws -> [GemFeeAsset] {
        try confirm.feeAssets(walletId: walletId, chain: chain)
    }

    public func simulation(inputType: GemTransactionInputType, simulation: SimulationResult?) throws -> GemConfirmSimulation {
        try confirm.simulation(inputType: inputType, simulation: simulation)
    }

    public func preload(walletId: WalletId, input: GemConfirmInput, options: GemConfirmLoadOptions) async throws -> GemConfirmPreload {
        try await confirm.preload(walletId: walletId, input: input, options: options)
    }

    public func execute(input: GemSendInput, signer: any GemTransactionSigner) async throws -> GemExecuteResult {
        try await confirm.execute(input: input, signer: signer)
    }

    public func syncMissingAssets(assetIds: [AssetId]) async throws -> [AssetId] {
        try await assets.syncMissingAssets(assetIds: assetIds)
    }

    public func trackPending() async throws {
        try await transactionState.trackPending()
    }

    public func track(walletId: WalletId, transactions: [Transaction]) async throws {
        try await transactionState.track(walletId: walletId, transactions: transactions)
    }

    public func addressUrl(chain: Chain, address: String) -> GemBlockExplorerLink {
        GemBlockExplorerLink(name: "Explorer", link: "https://explorer.test/\(chain)/\(address)")
    }

    public func addressName(chain: Chain, address: String) throws -> AddressName? {
        try names.addressName(chain: chain, address: address)
    }

    public func addressNames(requests: [ChainAddress]) async throws -> [AddressName] {
        try await names.getAddressNames(requests: requests)
    }

    public func defaultFeePriority(inputType: GemTransactionInputType) -> FeePriority {
        feeService.defaultPriority(inputType: inputType)
    }

    public func isInsufficientNetworkFee(feeAssetId: AssetId, feeAvailable: String) -> Bool {
        feeService.isInsufficientNetworkFee(feeAssetId: feeAssetId, feeAvailable: feeAvailable)
    }

    public func transactionType(inputType: GemTransactionInputType) -> TransactionType {
        transferService.transactionType(inputType: inputType)
    }

    public func autocloseSummary(data: PerpetualModifyConfirmData) -> GemAutocloseSummary? {
        nil
    }

    public func applicationShortName(inputType: GemTransactionInputType) -> String? {
        if case let .generic(_, metadata, _) = inputType {
            return applicationMetadataService.shortName(metadata: metadata)
        }
        return nil
    }

    public func recentActivity(inputType: GemTransactionInputType) -> GemRecentActivity? {
        transferService.recentActivity(inputType: inputType)
    }

    public func outputAction(inputType: GemTransactionInputType) -> TransferDataOutputAction {
        transferService.output(inputType: inputType).outputAction
    }

    public func acquireAssetFlow(chain: Chain) -> GemAcquireAssetFlow {
        assetConfig.acquireFlow(chain: chain)
    }

    public func fee() -> GemFeeService { feeService }

    public func swapQuote() -> GemSwapQuoteService { swapQuoteService }


}

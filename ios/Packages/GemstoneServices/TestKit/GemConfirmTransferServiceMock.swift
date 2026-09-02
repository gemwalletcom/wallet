// Copyright (c). Gem Wallet. All rights reserved.

public import enum Gemstone.GemKeystoreAuthentication
public import protocol Gemstone.GemConfirmTransferServiceProtocol
public import protocol Gemstone.GemConfirmServiceProtocol
public import protocol Gemstone.GemNameServiceProtocol
public import protocol Gemstone.GemTransactionStateServiceProtocol
public import protocol Gemstone.GemTransactionSigner
public import struct Gemstone.GemBlockExplorerLink
public import struct Gemstone.GemConfirmInput
public import struct Gemstone.GemConfirmLoadOptions
public import struct Gemstone.GemConfirmMetadata
public import struct Gemstone.GemConfirmPreload
public import struct Gemstone.GemConfirmSceneLoad
public import struct Gemstone.GemConfirmSceneState
public import struct Gemstone.GemConfirmSimulationState
public import struct Gemstone.GemFeeAsset
public import struct Gemstone.GemSendInput
public import struct Gemstone.GemAutocloseSummary
public import enum Gemstone.GemExecuteResult
public import enum Gemstone.GemTransactionInputType
public import enum Gemstone.GemAcquireAssetFlow
public import class Gemstone.GemAssetConfigService
public import class Gemstone.GemSwapQuoteService
public import typealias Gemstone.AddressName
public import typealias Gemstone.Chain
public import typealias Gemstone.PerpetualModifyConfirmData
public import typealias Gemstone.SimulationResult
public import typealias Gemstone.Transaction
public import typealias Gemstone.WalletId
import Foundation
import GemstonePrimitives
import GemstonePrimitivesTestKit

public final class GemConfirmTransferServiceMock: GemConfirmTransferServiceProtocol, @unchecked Sendable {
    private let confirm: any GemConfirmServiceProtocol
    private let names: any GemNameServiceProtocol
    private let transactionState: any GemTransactionStateServiceProtocol
    private let signer: any GemTransactionSigner
    private let authenticationValue: GemKeystoreAuthentication
    private let assetConfig = GemAssetConfigService()
    private let swapQuoteService = GemSwapQuoteService()

    public init(
        confirm: any GemConfirmServiceProtocol = GemConfirmServiceMock(),
        names: any GemNameServiceProtocol = GemNameServiceMock(),
        transactionState: any GemTransactionStateServiceProtocol = GemTransactionStateServiceMock(),
        signer: any GemTransactionSigner = GemTransactionSignerMock(),
        authentication: GemKeystoreAuthentication = .none,
    ) {
        self.confirm = confirm
        self.names = names
        self.transactionState = transactionState
        self.signer = signer
        self.authenticationValue = authentication
    }

    public func authentication() -> GemKeystoreAuthentication {
        authenticationValue
    }

    public func metadata(walletId: WalletId, inputType: GemTransactionInputType) throws -> GemConfirmMetadata {
        try confirm.metadata(
            walletId: walletId,
            assetId: inputType.transactionAsset().id,
            feeAssetId: inputType.feeAsset().id,
            extraAssetIds: inputType.assetIds(),
        )
    }

    public func sceneState(walletId: WalletId, inputType: GemTransactionInputType, simulation result: SimulationResult?) -> GemConfirmSceneState {
        GemConfirmSceneState(
            feePriority: inputType.defaultFeePriority(),
            feeAsset: inputType.feeAsset(),
            metadata: try? metadata(walletId: walletId, inputType: inputType),
            simulation: try? confirm.simulation(inputType: inputType, simulation: result),
        )
    }

    public func feeAssets(walletId: WalletId, chain: Chain) throws -> [GemFeeAsset] {
        try confirm.feeAssets(walletId: walletId, chain: chain)
    }

    public func preload(walletId: WalletId, input: GemConfirmInput, options: GemConfirmLoadOptions) async throws -> GemConfirmPreload {
        try await confirm.preload(walletId: walletId, input: input, options: options)
    }

    public func loadScene(
        walletId: WalletId,
        input: GemConfirmInput,
        options: GemConfirmLoadOptions,
        simulation: SimulationResult?,
    ) async throws -> GemConfirmSceneLoad {
        let preload = try await preload(walletId: walletId, input: input, options: options)
        return GemConfirmSceneLoad(
            feeAssets: try feeAssets(walletId: walletId, chain: input.transfer.inputType.chain.rawValue),
            preload: preload,
            simulation: await simulationState(
                inputType: input.transfer.inputType,
                simulation: simulation ?? preload.confirmData.simulation,
            ),
        )
    }

    public func execute(input: GemSendInput) async throws -> GemExecuteResult {
        try await confirm.execute(input: input, signer: signer)
    }

    public func simulationState(inputType: GemTransactionInputType, simulation: SimulationResult?) async -> GemConfirmSimulationState {
        GemConfirmSimulationState(
            simulation: try? confirm.simulation(inputType: inputType, simulation: simulation),
            addressNames: [],
        )
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

    public func autocloseSummary(data: PerpetualModifyConfirmData) -> GemAutocloseSummary? {
        nil
    }

    public func acquireAssetFlow(chain: Chain) -> GemAcquireAssetFlow {
        assetConfig.acquireFlow(chain: chain)
    }

    public func swapQuote() -> GemSwapQuoteService { swapQuoteService }
}

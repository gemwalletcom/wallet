// Copyright (c). Gem Wallet. All rights reserved.

public import typealias Gemstone.AddressName
public import typealias Gemstone.Chain
public import typealias Gemstone.Currency
public import enum Gemstone.GemAcquireAssetFlow
public import class Gemstone.GemAssetConfigService
public import struct Gemstone.GemAutocloseSummary
public import typealias Gemstone.GemBigInt
public import struct Gemstone.GemBlockExplorerLink
public import struct Gemstone.GemConfirmData
public import struct Gemstone.GemConfirmInput
public import struct Gemstone.GemConfirmLoad
public import struct Gemstone.GemConfirmLoadOptions
public import struct Gemstone.GemConfirmMetadata
public import struct Gemstone.GemConfirmSimulationState
public import protocol Gemstone.GemConfirmTransferServiceProtocol
public import enum Gemstone.GemExecuteResult
public import enum Gemstone.GemKeystoreAuthentication
public import enum Gemstone.GemTransactionInputType
public import protocol Gemstone.GemTransactionSigner
public import protocol Gemstone.GemTransactionStateServiceProtocol
public import struct Gemstone.GemTransferData
public import typealias Gemstone.PerpetualModifyConfirmData
public import typealias Gemstone.SimulationResult
public import typealias Gemstone.Transaction
import Foundation
import GemstonePrimitives
import GemstonePrimitivesTestKit
import Primitives
import PrimitivesTestKit

public final class GemConfirmTransferServiceMock: GemConfirmTransferServiceProtocol, @unchecked Sendable {
    private let confirm: GemConfirmServiceMock
    private let loadResult: Result<GemConfirmLoad, any Error>
    private let transactionState: any GemTransactionStateServiceProtocol
    private let signer: any GemTransactionSigner
    private let authenticationValue: GemKeystoreAuthentication
    private let wallet: Wallet
    private let assetConfig = GemAssetConfigService()

    public init(
        wallet: Wallet = .mock(),
        confirm: GemConfirmServiceMock = GemConfirmServiceMock(),
        load: Result<GemConfirmLoad, any Error> = .success(.mock()),
        transactionState: any GemTransactionStateServiceProtocol = GemTransactionStateServiceMock(),
        signer: any GemTransactionSigner = GemTransactionSignerMock(),
        authentication: GemKeystoreAuthentication = .none,
    ) {
        self.wallet = wallet
        self.confirm = confirm
        loadResult = load
        self.transactionState = transactionState
        self.signer = signer
        self.authenticationValue = authentication
    }

    public func getCurrency() -> Currency {
        Primitives.Currency.usd.rawValue
    }

    public func authentication() -> GemKeystoreAuthentication {
        authenticationValue
    }

    public func confirmInput(wallet _: String, transfer: GemTransferData) throws -> GemConfirmInput {
        try GemConfirmInput(from: wallet.account(for: transfer.chain).map(), transfer: transfer)
    }

    public func initialState(transfer: GemTransferData, simulation _: SimulationResult?) async throws -> GemConfirmLoad {
        GemConfirmLoad(
            feeAsset: transfer.inputType.transactionAsset(),
            metadata: .mock(),
            feeAssets: [],
            simulation: GemConfirmSimulationState(chain: Primitives.Chain.ethereum.rawValue, result: nil, warnings: [], simulation: confirm.simulation, addressNames: []),
            addressName: nil,
            preload: nil,
        )
    }

    public func load(input _: GemConfirmInput, options _: GemConfirmLoadOptions, simulation _: SimulationResult?) async throws -> GemConfirmLoad {
        try loadResult.get()
    }

    public func execute(confirm data: GemConfirmData, value _: GemBigInt, networkFee _: GemBigInt, simulation _: SimulationResult?) async throws -> GemExecuteResult {
        try await confirm.execute(confirm: data, signer: signer)
    }

    public func trackPending() async throws {
        try await transactionState.trackPending()
    }

    public func addressUrl(chain: Chain, address: String) -> GemBlockExplorerLink {
        GemBlockExplorerLink(name: "Explorer", link: "https://explorer.test/\(chain)/\(address)")
    }

    public func autocloseSummary(data _: PerpetualModifyConfirmData) -> GemAutocloseSummary? {
        nil
    }

    public func acquireAssetFlow(chain: Chain) -> GemAcquireAssetFlow {
        assetConfig.acquireFlow(chain: chain)
    }
}

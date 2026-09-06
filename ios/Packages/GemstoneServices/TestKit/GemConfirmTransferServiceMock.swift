// Copyright (c). Gem Wallet. All rights reserved.

public import struct Gemstone.AddressName
public import typealias Gemstone.Chain
public import typealias Gemstone.Currency
public import enum Gemstone.GemAcquireAssetFlow
public import class Gemstone.GemAssetConfigService
public import struct Gemstone.GemAutocloseSummary
public import typealias Gemstone.GemBigInt
public import struct Gemstone.BlockExplorerLink
public import struct Gemstone.GemConfirmData
public import class Gemstone.GemConfirmSession
public import protocol Gemstone.GemConfirmTransferServiceProtocol
public import enum Gemstone.GemExecuteResult
public import enum Gemstone.GemKeystoreAuthentication
public import enum Gemstone.TransactionInputType
public import protocol Gemstone.GemTransactionSigner
public import protocol Gemstone.GemTransactionStateServiceProtocol
public import struct Gemstone.GemTransferData
public import typealias Gemstone.PerpetualModifyConfirmData
public import typealias Gemstone.SimulationResult
public import typealias Gemstone.Transaction
public import struct Gemstone.Wallet
import Foundation
import GemstonePrimitives
import GemstonePrimitivesTestKit
import Primitives
import PrimitivesTestKit

public final class GemConfirmTransferServiceMock: GemConfirmTransferServiceProtocol, @unchecked Sendable {
    private let confirm: GemConfirmServiceMock
    private let transactionState: any GemTransactionStateServiceProtocol
    private let signer: any GemTransactionSigner
    private let authenticationValue: GemKeystoreAuthentication
    private let wallet: Primitives.Wallet
    private let assetConfig = GemAssetConfigService()

    public init(
        wallet: Primitives.Wallet = .mock(),
        confirm: GemConfirmServiceMock = GemConfirmServiceMock(),
        transactionState: any GemTransactionStateServiceProtocol = GemTransactionStateServiceMock(),
        signer: any GemTransactionSigner = GemTransactionSignerMock(),
        authentication: GemKeystoreAuthentication = .none,
    ) {
        self.wallet = wallet
        self.confirm = confirm
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

    public func session(wallet _: Gemstone.Wallet, transfer _: GemTransferData, simulation _: SimulationResult?) -> GemConfirmSession {
        fatalError("not used")
    }

    public func execute(confirm data: GemConfirmData, value _: GemBigInt, networkFee _: GemBigInt, simulation _: SimulationResult?) async throws -> GemExecuteResult {
        try await confirm.execute(confirm: data, signer: signer)
    }

    public func trackPending() async throws {
        try await transactionState.trackPending()
    }

    public func addressUrl(chain: Chain, address: String) -> Gemstone.BlockExplorerLink {
        Gemstone.BlockExplorerLink(name: "Explorer", link: "https://explorer.test/\(chain)/\(address)")
    }

    public func autocloseSummary(data _: PerpetualModifyConfirmData) -> GemAutocloseSummary? {
        nil
    }

    public func acquireAssetFlow(chain: Chain) -> GemAcquireAssetFlow {
        assetConfig.acquireFlow(chain: chain)
    }
}

// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
@testable import Payments
import PaymentService
import PaymentServiceTestKit
import Primitives
import PrimitivesTestKit
import SimulationServiceTestKit
import Testing

@MainActor
struct PaymentActionExecutorTests {
    @Test
    func sendTransactionReturnsHash() async throws {
        let interactor = SigningRequestInteractableMock()
        interactor.transactionHash = "transaction-hash"

        let results = try await PaymentActionExecutor(interactor: interactor, simulator: SimulationServiceableMock(), assetsProvider: PaymentAssetsProvidableMock(assetsData: [.mock()])).perform(
            actions: [PaymentAction.sendTransaction(chain: .ethereum, transaction: .sui("transaction", .encodedTransaction))],
            paymentId: "pay_1",
            appMetadata: .mock(),
            payment: .mock(),
            wallet: .mock(),
        )

        #expect(results.results == ["transaction-hash"])
        #expect(results.transactionHash == "transaction-hash")
    }

    @Test
    func relayedPaymentHasNoTransactionOfItsOwn() async throws {
        let interactor = SigningRequestInteractableMock()
        interactor.signature = "permit-signature"

        let results = try await PaymentActionExecutor(interactor: interactor, simulator: SimulationServiceableMock(), assetsProvider: Self.approvalAssets()).perform(
            actions: [
                .approveToken(chain: .ethereum, approval: ApprovalData(token: "0xtoken", spender: "0xspender", value: "1", isUnlimited: true)),
                .mockSignMessage(data: Data("permit".utf8)),
            ],
            paymentId: "pay_1",
            appMetadata: .mock(),
            payment: Self.approvalPayment(),
            wallet: .mock(),
        )

        #expect(results.results == ["1", "permit-signature"])
        #expect(results.transactionHash == nil)
    }

    @Test
    func performsEveryActionInOrder() async throws {
        let interactor = SigningRequestInteractableMock()
        interactor.signature = "permit-signature"
        interactor.transactionHash = "approval-hash"

        let results = try await PaymentActionExecutor(interactor: interactor, simulator: SimulationServiceableMock(), assetsProvider: PaymentAssetsProvidableMock(assetsData: [.mock()])).perform(
            actions: [
                .sendTransaction(chain: .ethereum, transaction: .sui("approval", .encodedTransaction)),
                .mockSignMessage(data: Data("permit".utf8)),
            ],
            paymentId: "pay_1",
            appMetadata: .mock(),
            payment: .mock(),
            wallet: .mock(),
        )

        #expect(results.results == ["approval-hash", "permit-signature"])
        #expect(interactor.signMessagePayloads.count == 1)
    }

    @Test
    func signMessageCarriesPaymentAmount() async throws {
        let interactor = SigningRequestInteractableMock()
        let payment = PaymentData.mock(quote: .mock(amount: .mock(value: "25000", symbol: "USDT")))

        _ = try await PaymentActionExecutor(interactor: interactor, simulator: SimulationServiceableMock(), assetsProvider: PaymentAssetsProvidableMock(assetsData: [.mock()])).perform(
            actions: [PaymentAction.mockSignMessage(data: Data("pay".utf8))],
            paymentId: "pay_1",
            appMetadata: .mock(),
            payment: payment,
            wallet: .mock(),
        )

        #expect(interactor.signMessagePayloads.first?.payment == payment)
    }

    @Test
    func paymentIsRecordedOnlyAfterEveryActionIsSubmitted() async throws {
        let interactor = SigningRequestInteractableMock()
        var approvalsWhenRecorded: Int?
        var signaturesWhenRecorded: Int?

        _ = try await PaymentActionExecutor(
            interactor: interactor,
            simulator: SimulationServiceableMock(),
            assetsProvider: Self.approvalAssets(),
        ).perform(
            actions: [
                .approveToken(chain: .ethereum, approval: .mock()),
                .mockSignMessage(data: Data("permit".utf8)),
            ],
            paymentId: "pay_1",
            appMetadata: .mock(),
            payment: Self.approvalPayment(),
            wallet: .mock(),
            onSubmitted: {
                approvalsWhenRecorded = interactor.sentTransferData.count
                signaturesWhenRecorded = interactor.signMessagePayloads.count
            },
        )

        #expect(approvalsWhenRecorded == 1)
        #expect(signaturesWhenRecorded == 1)
    }

    @Test
    func signMessageIsSimulatedBeforeItIsShown() async throws {
        let interactor = SigningRequestInteractableMock()
        let warning = SimulationWarning(severity: .critical, warning: .suspiciousSpender, message: "suspicious spender")
        let simulator = SimulationServiceableMock(
            result: SimulationResult(warnings: [warning], balanceChanges: [], payload: [], header: .none),
        )

        _ = try await PaymentActionExecutor(interactor: interactor, simulator: simulator, assetsProvider: PaymentAssetsProvidableMock(assetsData: [.mock()])).perform(
            actions: [PaymentAction.mockSignMessage(data: Data("{}".utf8))],
            paymentId: "pay_1",
            appMetadata: .mock(),
            payment: .mock(),
            wallet: .mock(),
        )

        #expect(interactor.signMessagePayloads.first?.simulation.warnings == [warning])
    }

    private static let approvalAssetId = AssetId(chain: .ethereum, tokenId: "0xtoken")

    private static func approvalPayment() -> PaymentData {
        .mock(quote: .mock(amount: .mock(assetId: approvalAssetId)))
    }

    private static func approvalAssets() -> PaymentAssetsProvidableMock {
        PaymentAssetsProvidableMock(assetsData: [.mock(asset: .mock(id: approvalAssetId))])
    }
}

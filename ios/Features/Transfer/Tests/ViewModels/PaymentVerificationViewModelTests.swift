// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.PaymentInvoice
import GemstonePrimitives
import GemstonePrimitivesTestKit
import GemstoneServicesTestKit
import Primitives
import PrimitivesTestKit
import Testing
@testable import Transfer

@MainActor
struct PaymentVerificationViewModelTests {
    @Test
    func completedFormReselectsTheAssetAndHandsOverTheConfirm() async throws {
        let invoice = PaymentInvoice.mock(quotes: [.mock(asset: .mockEthereum()), .mock(asset: .mockBNB())])
        let service = GemPaymentServiceMock(load: .success(.sign(transfer: .mockPayment(asset: .mockBNB(), invoice: invoice))))
        var destination: PaymentDestination?
        let model = try PaymentVerificationViewModel.mock(invoice: invoice, service: service) { destination = $0 }

        await model.verified()

        #expect(service.selectedAssetIds == [Asset.mockBNB().id.identifier])
        guard case let .confirm(transfer)? = destination else {
            Issue.record("Expected a confirm destination")
            return
        }
        #expect(transfer.chain == .smartChain)
    }

    @Test
    func onlyTheCompleteMessageCounts() async throws {
        let service = GemPaymentServiceMock(load: .success(.sign(transfer: .mockPayment())))
        var destination: PaymentDestination?
        let model = try PaymentVerificationViewModel.mock(service: service) { destination = $0 }

        model.onMessage(["type": "IC_ERROR"])
        await Task.yield()

        #expect(service.selectedAssetIds.isEmpty)
        #expect(destination == nil)
    }

    @Test
    func failedReselectShowsTheError() async throws {
        let model = try PaymentVerificationViewModel.mock(service: GemPaymentServiceMock(load: .failure(AnyError("gateway")))) { _ in }

        await model.verified()

        #expect(model.isPresentingAlertMessage?.message == "gateway")
    }
}

private extension PaymentVerificationViewModel {
    static func mock(
        invoice: PaymentInvoice = .mock(),
        service: GemPaymentServiceMock,
        onComplete: @escaping (PaymentDestination) -> Void,
    ) throws -> PaymentVerificationViewModel {
        PaymentVerificationViewModel(
            verification: try PaymentVerification(url: "https://walletconnect.com/collect", invoice: invoice, assetId: Asset.mockBNB().id),
            wallet: .mock(accounts: [.mock(chain: .ethereum), .mock(chain: .smartChain)]),
            service: service,
            onComplete: onComplete,
        )
    }
}

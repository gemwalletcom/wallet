// Copyright (c). Gem Wallet. All rights reserved.

import Components
import enum Gemstone.GemServiceError
import struct Gemstone.LightningInvoice
import GemstonePrimitivesTestKit
import GemstoneServices
import GemstoneServicesTestKit
import Localization
import Primitives
import PrimitivesTestKit
import Testing
@testable import Transfer
import TransferTestKit

@MainActor
struct LightningReceiveSceneViewModelTests {
    @Test
    func openShowsTheAmountEntry() async {
        let wallet = Wallet.mock(accounts: [.mock(chain: .lightning)])
        let service = GemLightningServiceMock()
        let model = LightningReceiveSceneViewModel.mock(service: service, wallet: wallet)

        await model.open()

        #expect(model.state == .amount)
        #expect(service.openCalls == [wallet.id])
    }

    @Test
    func createInvoiceOpensTheSessionAndUsesTheAmount() async {
        let service = GemLightningServiceMock()
        let model = LightningReceiveSceneViewModel.mock(service: service)
        await model.open()

        model.amount = "1500"
        await model.onCreateInvoice()

        #expect(service.openCalls.count == 2)
        #expect(service.wallet.createInvoiceAmounts == [1500])
        #expect(model.state == .invoice(.mock(invoice: "lnbc1invoice", amountSats: 1500, paymentHash: "payment-hash", expiresAt: 4_102_444_800)))
    }

    @Test
    func anEmptyAmountCreatesAnAnyAmountInvoice() async {
        let service = GemLightningServiceMock()
        let model = LightningReceiveSceneViewModel.mock(service: service)

        await model.onCreateInvoice()

        #expect(service.wallet.createInvoiceAmounts == [nil])
        #expect(model.invoiceAmountText(service.wallet.invoice) == Localized.Receive.lightningAnyAmount)
    }

    @Test
    func aReceivedPaymentSyncsTheWalletAndCompletes() async {
        let wallet = Wallet.mock(accounts: [.mock(chain: .lightning)])
        let service = GemLightningServiceMock()
        let invoice = service.wallet.invoice
        service.wallet.payments = [.mock(status: .completed, invoice: invoice.invoice)]
        var messages: [String] = []
        let model = LightningReceiveSceneViewModel.mock(service: service, wallet: wallet, onComplete: { messages.append($0.title) })

        await model.waitForPayment(invoice)

        #expect(messages == [Localized.Receive.lightningPaymentReceived])
        #expect(service.wallet.syncCallsCount == 1)
        #expect(service.syncCalls == [wallet.id])
    }

    @Test
    func anExpiredInvoiceStopsWaiting() async {
        let invoice = LightningInvoice.mock(invoice: "lnbc1expired")
        let service = GemLightningServiceMock()
        var messages: [String] = []
        let model = LightningReceiveSceneViewModel.mock(service: service, onComplete: { messages.append($0.title) })

        await model.waitForPayment(invoice)

        #expect(model.state == .expired(invoice))
        #expect(messages.isEmpty)
    }

    @Test
    func aPaymentThatLandedBeforeExpiryCompletesInsteadOfExpiring() async {
        let invoice = LightningInvoice.mock(invoice: "lnbc1expired")
        let service = GemLightningServiceMock()
        service.wallet.payments = [.mock(status: .completed, invoice: invoice.invoice)]
        var messages: [String] = []
        let model = LightningReceiveSceneViewModel.mock(service: service, onComplete: { messages.append($0.title) })
        await model.open()

        await model.waitForPayment(invoice)

        #expect(model.state == .amount)
        #expect(messages == [Localized.Receive.lightningPaymentReceived])
    }

    @Test
    func changeAmountReturnsToTheAmountEntry() async {
        let model = LightningReceiveSceneViewModel.mock()
        await model.onCreateInvoice()

        model.onChangeAmount()

        #expect(model.state == .amount)
        #expect(model.renderedImage == nil)
    }

    @Test
    func aLockedSessionOffersUnlockAndOpensAfterIt() async {
        let service = GemLightningServiceMock()
        let biometry = BiometryAuthenticationMock()
        service.openError = GemServiceError.Cancelled
        let model = LightningReceiveSceneViewModel.mock(service: service, biometry: biometry)

        await model.open()

        #expect(model.state == .unlockRequired)
        #expect(model.unavailableMessage == Localized.Receive.lightningUnlockRequired)
        #expect(model.retryTitle == Localized.Lock.unlock)

        service.openError = nil
        await model.onRetry()

        #expect(biometry.authenticateCallsCount == 1)
        #expect(model.state == .amount)
    }

    @Test
    func aCancelledUnlockStaysOnTheUnlockScreen() async {
        let service = GemLightningServiceMock()
        let biometry = BiometryAuthenticationMock()
        biometry.authenticateError = BiometryAuthenticationError.cancelledByUser
        service.openError = GemServiceError.Cancelled
        let model = LightningReceiveSceneViewModel.mock(service: service, biometry: biometry)
        await model.open()

        await model.onRetry()

        #expect(model.state == .unlockRequired)
        #expect(service.openCalls.count == 1)
    }

    @Test
    func aLockedSessionKeepsTheInvoiceAndOffersUnlock() async {
        let service = GemLightningServiceMock()
        let biometry = BiometryAuthenticationMock()
        let model = LightningReceiveSceneViewModel.mock(service: service, biometry: biometry)
        await model.onCreateInvoice()
        let invoice = service.wallet.invoice

        service.openError = GemServiceError.Cancelled
        #expect(await model.checkPayment(invoice) == false)

        #expect(model.state == .invoice(invoice))
        #expect(model.isUnlockRequired)

        await model.onUnlock()
        service.openError = nil

        #expect(biometry.authenticateCallsCount == 1)
        #expect(!model.isUnlockRequired)
        #expect(await model.checkPayment(invoice) == false)
        #expect(!model.isUnlockRequired)
    }

    @Test
    func aCancelledUnlockKeepsOfferingIt() async {
        let service = GemLightningServiceMock()
        let biometry = BiometryAuthenticationMock()
        biometry.authenticateError = BiometryAuthenticationError.cancelledByUser
        let model = LightningReceiveSceneViewModel.mock(service: service, biometry: biometry)
        service.openError = GemServiceError.Cancelled
        _ = await model.checkPayment(service.wallet.invoice)

        await model.onUnlock()

        #expect(model.isUnlockRequired)
    }

    @Test
    func retryReturnsToTheLastUnexpiredInvoice() async {
        let service = GemLightningServiceMock()
        let model = LightningReceiveSceneViewModel.mock(service: service)
        await model.onCreateInvoice()

        await model.open()

        #expect(model.state == .invoice(service.wallet.invoice))
    }

    @Test
    func aFailedInvoiceStaysOnTheAmountEntry() async {
        let service = GemLightningServiceMock()
        service.wallet.createInvoiceError = AnyError("offline")
        let model = LightningReceiveSceneViewModel.mock(service: service)
        await model.open()

        await model.onCreateInvoice()

        #expect(model.state == .amount)
        #expect(model.isPresentingToastMessage?.title == Localized.Errors.errorOccurred)
    }

    @Test
    func otherOpenFailuresShowLightningUnavailable() async {
        let service = GemLightningServiceMock()
        service.openError = AnyError("offline")
        let model = LightningReceiveSceneViewModel.mock(service: service)

        await model.open()

        #expect(model.state == .unavailable)
        #expect(model.unavailableMessage == Localized.Receive.lightningUnavailable)
        #expect(model.retryTitle == Localized.Common.tryAgain)
    }
}

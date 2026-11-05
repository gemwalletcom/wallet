// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemCopy
import protocol Gemstone.GemLightningServiceProtocol
import enum Gemstone.GemServiceError
import struct Gemstone.LightningInvoice
import GemstonePrimitives
import GemstoneServices
import Localization
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

@Observable
@MainActor
public final class LightningReceiveSceneViewModel: Sendable {
    private static let pollInterval: Duration = .seconds(4)
    private static let paymentsLimit: UInt64 = 20
    private static let invoiceScheme = "lightning:"

    private let service: any GemLightningServiceProtocol
    private let biometry: any BiometryAuthenticatable
    private let wallet: Wallet
    private let onComplete: (ToastMessage) -> Void
    private let generator = QRCodeGenerator()
    private var lastInvoice: LightningInvoice?

    let asset = Chain.lightning.asset

    var state: LightningReceiveState = .opening
    var amount = ""
    var renderedImage: UIImage?
    var isCreatingInvoice = false
    var isPresentingCopyToast = false
    var isPresentingToastMessage: ToastMessage?
    var isUnlockRequired = false

    public init(service: any GemLightningServiceProtocol, biometry: any BiometryAuthenticatable, wallet: Wallet, onComplete: @escaping (ToastMessage) -> Void) {
        self.service = service
        self.biometry = biometry
        self.wallet = wallet
        self.onComplete = onComplete
    }

    var title: String {
        Localized.Wallet.receive
    }

    var amountInputConfig: LightningAmountInputConfig {
        LightningAmountInputConfig(secondaryText: amountSats.flatMap(bitcoinText) ?? "")
    }

    var amountHint: String {
        Localized.Receive.lightningAmountHint
    }

    var createInvoiceTitle: String {
        Localized.Receive.lightningCreateInvoice
    }

    var createInvoiceButtonState: ButtonState {
        isCreatingInvoice ? .loading() : .normal
    }

    var changeAmountTitle: String {
        Localized.Receive.lightningChangeAmount
    }

    var copyTitle: String {
        Localized.Common.copy
    }

    var retryTitle: String {
        switch state {
        case .unlockRequired: unlockTitle
        case .opening, .amount, .invoice, .expired, .unavailable: Localized.Common.tryAgain
        }
    }

    var unlockTitle: String {
        Localized.Lock.unlock
    }

    var unlockHint: String {
        Localized.Receive.lightningUnlockRequired
    }

    var unavailableMessage: String {
        switch state {
        case .unlockRequired: Localized.Receive.lightningUnlockRequired
        case .opening, .amount, .invoice, .expired, .unavailable: Localized.Receive.lightningUnavailable
        }
    }

    var unavailableImage: String {
        switch state {
        case .unlockRequired: SystemImage.lock
        case .opening, .amount, .invoice, .expired, .unavailable: SystemImage.exclamationmarkTriangle
        }
    }

    func invoiceAmountText(_ invoice: LightningInvoice) -> String {
        invoice.amountSats.map { "\($0.formatted()) \(LightningAmountInputConfig.satsSymbol)" } ?? Localized.Receive.lightningAnyAmount
    }

    func invoiceHint(_ invoice: LightningInvoice) -> String {
        Localized.Receive.lightningInvoiceHint(TransactionDateFormatter(date: invoice.expiryDate).row)
    }

    var expiredText: String {
        Localized.Transaction.Status.expired
    }

    func invoiceText(_ invoice: LightningInvoice) -> String {
        guard invoice.invoice.count > 24 else { return invoice.invoice }
        return "\(invoice.invoice.prefix(12))…\(invoice.invoice.suffix(8))"
    }

    func copy(_ invoice: LightningInvoice) -> GemCopy {
        GemCopy(kind: .plain, value: invoice.invoice, display: Localized.Transfer.Recipient.lightningInvoice)
    }

    private var amountSats: UInt64? {
        UInt64(amount).flatMap { $0 > 0 ? $0 : nil }
    }

    private func bitcoinText(_ sats: UInt64) -> String? {
        NumberInput.format().inputText(value: String(sats), decimals: UInt32(asset.decimals)).map { "\($0) \(asset.symbol)" }
    }
}

// MARK: - Business Logic

extension LightningReceiveSceneViewModel {
    func open() async {
        state = .opening
        do {
            _ = try await service.open(walletId: wallet.id)
            state = lastInvoice.map(invoiceState) ?? .amount
        } catch {
            state = failureState(error)
        }
    }

    func onCreateInvoice() async {
        guard !isCreatingInvoice else { return }
        isCreatingInvoice = true
        defer { isCreatingInvoice = false }
        do {
            let lightning = try await service.open(walletId: wallet.id)
            let invoice = try await lightning.createInvoice(amountSats: amountSats, expirySecs: nil)
            renderedImage = await generator.generate(from: Self.invoiceScheme + invoice.invoice, size: CGSize(width: ReceiveQRCodeCard.size, height: ReceiveQRCodeCard.size))
            lastInvoice = invoice
            state = .invoice(invoice)
        } catch GemServiceError.Cancelled {
            state = .unlockRequired
        } catch {
            debugLog("lightning receive: create invoice failed: \(error)")
            isPresentingToastMessage = .error(error)
        }
    }

    func onChangeAmount() {
        lastInvoice = nil
        renderedImage = nil
        state = .amount
    }

    func onCopyInvoice() {
        isPresentingCopyToast = true
    }

    func waitForPayment(_ invoice: LightningInvoice) async {
        while !Task.isCancelled {
            guard await !checkPayment(invoice) else { return }
            try? await Task.sleep(for: Self.pollInterval)
        }
    }

    func checkPayment(_ invoice: LightningInvoice) async -> Bool {
        let isExpired = Date.now >= invoice.expiryDate
        do {
            let isPaid = try await isPaid(invoice)
            isUnlockRequired = false
            if isPaid {
                await completePayment()
                return true
            }
        } catch GemServiceError.Cancelled {
            isUnlockRequired = true
        } catch {
            debugLog("lightning receive: payment check failed: \(error)")
        }
        guard isExpired else { return false }
        state = .expired(invoice)
        return true
    }

    func onUnlock() async {
        do {
            try await biometry.authenticate(reason: unlockHint)
        } catch {
            debugLog("lightning receive: unlock failed: \(error)")
            return
        }
        isUnlockRequired = false
        if state == .unlockRequired {
            await open()
        }
    }

    func onRetry() async {
        switch state {
        case .unlockRequired: await onUnlock()
        case .opening, .amount, .invoice, .expired, .unavailable: await open()
        }
    }
}

// MARK: - Private

extension LightningReceiveSceneViewModel {
    private func isPaid(_ invoice: LightningInvoice) async throws -> Bool {
        let lightning = try await service.open(walletId: wallet.id)
        try await lightning.sync()
        return try await lightning.payments(limit: Self.paymentsLimit, offset: 0).contains {
            $0.direction == .incoming && $0.status == .completed && $0.invoice == invoice.invoice
        }
    }

    private func completePayment() async {
        do {
            try await service.sync(walletId: wallet.id)
        } catch {
            debugLog("lightning receive: wallet sync failed: \(error)")
        }
        onComplete(.success(Localized.Receive.lightningPaymentReceived))
    }

    private func invoiceState(_ invoice: LightningInvoice) -> LightningReceiveState {
        Date.now < invoice.expiryDate ? .invoice(invoice) : .expired(invoice)
    }

    private func failureState(_ error: any Error) -> LightningReceiveState {
        debugLog("lightning receive: \(error)")
        return (error as? GemServiceError) == .Cancelled ? .unlockRequired : .unavailable
    }
}

// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemLightningReceive
import protocol Gemstone.GemLightningServiceProtocol
import enum Gemstone.GemServiceError
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
    private static let amountDebounce: Duration = .milliseconds(600)

    private let service: any GemLightningServiceProtocol
    private let biometry: any BiometryAuthenticatable
    private let walletId: WalletId
    private let onComplete: (ToastMessage) -> Void
    private let generator = QRCodeGenerator()

    var state: LightningReceiveState = .loading
    var amount = ""
    var renderedImage: UIImage?
    var isPresentingCopyToast = false

    public init(service: any GemLightningServiceProtocol, biometry: any BiometryAuthenticatable, walletId: WalletId, onComplete: @escaping (ToastMessage) -> Void) {
        self.service = service
        self.biometry = biometry
        self.walletId = walletId
        self.onComplete = onComplete
    }

    var title: String {
        Localized.Wallet.receive
    }

    var copyTitle: String {
        Localized.Common.copy
    }

    var unlockTitle: String {
        Localized.Lock.unlock
    }

    var unavailableTitle: String {
        Localized.Common.notAvailable
    }

    var tryAgainTitle: String {
        Localized.Common.tryAgain
    }

    func amountInputConfig(_ receive: GemLightningReceive) -> AmountInputConfig {
        AmountInputConfig(field: receive.field, canSwitchInputType: false, numberFormat: NumberInput.format(), secondaryText: "", onTapActionButton: nil)
    }
}

// MARK: - Business Logic

extension LightningReceiveSceneViewModel {
    func onChangeAmount() async {
        if amount.isNotEmpty {
            try? await Task.sleep(for: Self.amountDebounce)
        }
        guard !Task.isCancelled else { return }
        await createInvoice()
    }

    func onRefresh() async {
        guard case let .ready(receive) = state else { return }
        do {
            switch try await service.receiveState(walletId: walletId, invoice: receive.invoice) {
            case .pending: break
            case .expired: await createInvoice()
            case .received: onComplete(.success(Localized.Transaction.Title.received))
            }
        } catch GemServiceError.Cancelled {
            state = .locked
        } catch {
            debugLog("lightning receive: payment check failed: \(error)")
        }
    }

    func onUnlock() async {
        do {
            try await biometry.authenticate(reason: unlockTitle)
        } catch {
            debugLog("lightning receive: unlock failed: \(error)")
            return
        }
        await createInvoice()
    }

    func onCopy() {
        isPresentingCopyToast = true
    }
}

// MARK: - Private

extension LightningReceiveSceneViewModel {
    private func createInvoice() async {
        do {
            let receive = try await service.receive(walletId: walletId, amount: amount, format: NumberInput.format())
            renderedImage = await generator.generate(from: receive.qr, size: CGSize(width: ReceiveQRCodeCard.size, height: ReceiveQRCodeCard.size))
            state = .ready(receive)
        } catch GemServiceError.Cancelled {
            state = .locked
        } catch {
            debugLog("lightning receive: \(error)")
            state = .unavailable
        }
    }
}

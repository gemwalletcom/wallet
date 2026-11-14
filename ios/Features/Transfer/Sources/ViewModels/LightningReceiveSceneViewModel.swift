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
    private let service: any GemLightningServiceProtocol
    private let walletId: WalletId
    private let onComplete: (ToastMessage) -> Void
    private let generator = QRCodeGenerator()

    var state: LightningReceiveState = .loading
    var amount = ""
    var renderedImage: UIImage?
    var isPresentingCopyToast = false

    public init(service: any GemLightningServiceProtocol, walletId: WalletId, onComplete: @escaping (ToastMessage) -> Void) {
        self.service = service
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

    var loadTrigger: LightningReceiveTrigger {
        LightningReceiveTrigger(amount: amount)
    }

    func amountInputConfig(_ receive: GemLightningReceive) -> AmountInputConfig {
        AmountInputConfig(field: receive.field, canSwitchInputType: false, numberFormat: NumberInput.format(), secondaryText: "", onTapActionButton: nil)
    }
}

// MARK: - Business Logic

extension LightningReceiveSceneViewModel {
    func onLoad() async {
        do {
            let receive = try await service.receive(walletId: walletId, amount: amount, format: NumberInput.format())
            let image = await generator.generate(from: receive.qr, size: CGSize(width: ReceiveQRCodeCard.size, height: ReceiveQRCodeCard.size))
            guard !Task.isCancelled else { return }
            renderedImage = image
            state = .ready(receive)
        } catch GemServiceError.Cancelled {
            state = .locked
        } catch {
            debugLog("lightning receive: \(error)")
            state = .unavailable
        }
    }

    func onRefresh() async {
        guard case let .ready(receive) = state else { return }
        do {
            let receiveState = try await service.receiveState(walletId: walletId, invoice: receive.copy.value)
            guard state == .ready(receive) else { return }
            switch receiveState {
            case .pending: break
            case .expired: await onLoad()
            case .received: onComplete(.success(Localized.Transaction.Title.received))
            }
        } catch GemServiceError.Cancelled {
            state = .locked
        } catch {
            debugLog("lightning receive: payment check failed: \(error)")
        }
    }

    func onCopy() {
        isPresentingCopyToast = true
    }
}

// Copyright (c). Gem Wallet. All rights reserved.

import EventPresenterService
import Components
import PrimitivesComponents
import Foundation
import Localization
import Primitives
import Style

public final class PaymentLinkManager: PaymentLinkPayable, Sendable {
    private let paymentManager: PaymentManager
    private let eventPresenterService: EventPresenterService

    public init(
        paymentManager: PaymentManager,
        eventPresenterService: EventPresenterService,
    ) {
        self.paymentManager = paymentManager
        self.eventPresenterService = eventPresenterService
    }

    @MainActor
    public func pay(link: PaymentLink, wallet: Wallet) async {
        guard !wallet.isViewOnly else {
            return present(error: Localized.Wallet.Watch.Tooltip.title)
        }
        eventPresenterService.toastPresenter.toastMessage = ToastMessage(
            title: Localized.Common.loading,
            image: SystemImage.network,
        )
        do {
            present(outcome: try await paymentManager.pay(link: link, wallet: wallet))
        } catch {
            present(error: error.localizedDescription)
        }
    }
}

// MARK: - Private

extension PaymentLinkManager {
    @MainActor
    private func present(outcome: PaymentOutcome) {
        switch outcome.status {
        case .succeeded:
            eventPresenterService.toastPresenter.toastMessage = ToastMessage(
                title: Localized.Transaction.Status.confirmed,
                image: SystemImage.checkmark,
            )
        case .processing:
            eventPresenterService.toastPresenter.toastMessage = ToastMessage(
                title: Localized.Transaction.Status.pending,
                image: SystemImage.refresh,
            )
        case .cancelled:
            return
        case .expired:
            present(error: Localized.Errors.paymentExpired)
        case .failed, .requiresAction:
            present(error: Localized.Transaction.Status.failed)
        }
    }

    @MainActor
    private func present(error: String) {
        debugLog("PaymentLinkManager payment error: \(error)")
        eventPresenterService.toastPresenter.toastMessage = .error(error)
    }
}

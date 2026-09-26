// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import GemstonePrimitives
import GemstoneServices
import Localization
import Primitives

@Observable
final class AcceptTermsSceneViewModel {
    private let preferences: ObservablePreferences
    let onNext: VoidAction

    init(preferences: ObservablePreferences, onNext: VoidAction) {
        self.preferences = preferences
        self.onNext = onNext
    }

    func accept() {
        preferences.acceptTerms()
        onNext?()
    }

    var termsAndServicesURL: URL {
        AppUrl.page(.termsOfService)
    }

    let title: String = Localized.Onboarding.AcceptTerms.title
    let message: String = Localized.Onboarding.AcceptTerms.message

    var items: [TermItemViewModel] = GemConstants.acceptTermsItems.map { TermItemViewModel(message: $0.message) }

    var isConfirmed: Bool {
        items.allSatisfy(\.isConfirmed)
    }

    var state: StateViewType<Bool> {
        isConfirmed ? .data(true) : .noData
    }
}

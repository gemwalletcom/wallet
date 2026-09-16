// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import func Gemstone.acceptTermsItems
import GemstonePrimitives
import Localization
import Primitives

@Observable
final class AcceptTermsViewModel {
    let onNext: VoidAction

    init(onNext: VoidAction) {
        self.onNext = onNext
    }

    var termsAndServicesURL: URL {
        AppUrl.page(.termsOfService)
    }

    let title: String = Localized.Onboarding.AcceptTerms.title
    let message: String = Localized.Onboarding.AcceptTerms.message

    var items: [TermItemViewModel] = acceptTermsItems().map { TermItemViewModel(message: $0.message) }

    var isConfirmed: Bool {
        items.allSatisfy(\.isConfirmed)
    }

    var state: StateViewType<Bool> {
        isConfirmed ? .data(true) : .noData
    }
}

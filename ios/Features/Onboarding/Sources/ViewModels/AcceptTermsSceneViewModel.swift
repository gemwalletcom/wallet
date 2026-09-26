// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import enum Gemstone.GemAcceptTermsItem
import struct Gemstone.GemTermsSession
import struct Gemstone.GemTermsViewState
import func Gemstone.newTermsSession
import GemstonePrimitives
import GemstoneServices
import Localization
import Primitives

@Observable
final class AcceptTermsSceneViewModel {
    private let preferences: ObservablePreferences
    private var session: GemTermsSession = newTermsSession()
    let onNext: VoidAction

    init(preferences: ObservablePreferences, onNext: VoidAction) {
        self.preferences = preferences
        self.onNext = onNext
    }

    func accept() {
        preferences.acceptTerms()
        onNext?()
    }

    func onToggle(_ item: GemAcceptTermsItem) {
        session = session.onToggle(item: item)
    }

    var termsAndServicesURL: URL {
        AppUrl.page(.termsOfService)
    }

    let title: String = Localized.Onboarding.AcceptTerms.title
    let message: String = Localized.Onboarding.AcceptTerms.message

    var viewState: GemTermsViewState {
        session.viewState()
    }

    var state: StateViewType<Bool> {
        viewState.isAccepted ? .data(true) : .noData
    }
}

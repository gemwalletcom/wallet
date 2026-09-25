// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemSlippageSelection
import struct Gemstone.GemSlippageSession
import struct Gemstone.GemSlippageSuggestion
import struct Gemstone.GemSlippageViewState
import func Gemstone.newSlippageSession
import GemstonePrimitives
import InfoSheet
import Localization
import Primitives
import PrimitivesComponents

@MainActor
@Observable
public final class SwapSlippageViewModel {
    private let onSelect: (GemSlippageSelection) -> Void
    private var session: GemSlippageSession
    private(set) var viewState: GemSlippageViewState

    var infoSheet: InfoSheetType?

    public init(chain: Chain, slippage: GemSlippageSelection, onSelect: @escaping (GemSlippageSelection) -> Void) {
        self.onSelect = onSelect
        let session = newSlippageSession(selection: slippage, chain: chain.rawValue, format: NumberInput.format())
        self.session = session
        viewState = session.viewState()
    }

    var title: String {
        Localized.Swap.slippage
    }

    var autoTitle: String {
        Localized.Swap.slippageAuto
    }

    var autoDescription: String {
        Localized.Swap.slippageAutoDescription
    }

    var placeholder: String {
        viewState.placeholder
    }

    var isAuto: Bool {
        get { viewState.isAuto }
        set { update(session.onAuto(isAuto: newValue)) }
    }

    var input: String {
        get { viewState.input }
        set { update(session.onInput(text: newValue)) }
    }

    var footerText: String? {
        viewState.footer?.text
    }

    var isConfirmEnabled: Bool {
        viewState.allowsConfirm
    }

    var suggestions: [GemSlippageSuggestion] {
        viewState.suggestions
    }

    func onSelect(suggestion: GemSlippageSuggestion) {
        input = suggestion.input
    }

    func onSelectInfo() {
        infoSheet = .slippage
    }

    func confirm() {
        onSelect(viewState.selection)
    }
}

// MARK: - Private

extension SwapSlippageViewModel {
    private func update(_ session: GemSlippageSession) {
        self.session = session
        viewState = session.viewState()
    }
}

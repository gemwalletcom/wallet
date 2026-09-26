// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import func Gemstone.emptyState
import struct Gemstone.GemEmptyState
import enum Gemstone.GemEmptyStateAction
import enum Gemstone.GemEmptyStateKind
import Localization
import Primitives
import Style
import SwiftUI

public struct EmptyStateViewModel: EmptyContentViewable {
    private let state: GemEmptyState
    private let symbol: String
    private let onAction: ((GemEmptyStateAction) -> Void)?

    public init(
        state: GemEmptyState,
        symbol: String = "",
        onAction: ((GemEmptyStateAction) -> Void)? = nil,
    ) {
        self.state = state
        self.symbol = symbol
        self.onAction = onAction
    }

    public init(kind: GemEmptyStateKind, symbol: String = "", onAction: ((GemEmptyStateAction) -> Void)? = nil) {
        self.init(state: emptyState(kind: kind), symbol: symbol, onAction: onAction)
    }

    public var title: String {
        state.title.text(symbol: symbol)
    }

    public var description: String? {
        state.description?.text(symbol: symbol)
    }

    public var image: Image? {
        state.image.image
    }

    public var buttons: [EmptyAction] {
        state.actions.map { action in
            EmptyAction(title: action.title, action: onAction.map { onAction in { onAction(action) } })
        }
    }
}

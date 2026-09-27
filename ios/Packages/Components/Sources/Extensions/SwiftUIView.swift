// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI

// MARK: - View builders

public extension View {
    @ViewBuilder func isVisible(_ isVisible: Bool) -> some View {
        if isVisible {
            self
        } else {
            hidden()
        }
    }

    func `if`(
        _ condition: Bool,
        content: (Self) -> some View,
    ) -> some View {
        ifElse(condition, ifContent: content, elseContent: { _ in self })
    }

    @ViewBuilder
    func ifElse(
        _ condition: Bool,
        ifContent: (Self) -> some View,
        elseContent: (Self) -> some View,
    ) -> some View {
        if condition {
            ifContent(self)
        } else {
            elseContent(self)
        }
    }

    @ViewBuilder
    func ifLet<Wrapped>(
        _ optional: Wrapped?,
        content: (Self, Wrapped) -> some View,
    ) -> some View {
        if let value = optional {
            content(self, value)
        } else {
            self
        }
    }
}

// MARK: - Sheet

public extension View {}

// MARK: - Syntactic sugar

public extension View {
    func frame(size: CGFloat, alignment: Alignment = .center) -> some View {
        frame(width: size, height: size, alignment: alignment)
    }
}

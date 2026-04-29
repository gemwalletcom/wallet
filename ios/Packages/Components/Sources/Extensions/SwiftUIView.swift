// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI

// MARK: - View builders

public extension View {
    @ViewBuilder func isHidden(_ isHidden: Bool) -> some View {
        if isHidden {
            hidden()
        } else {
            self
        }
    }

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

    @ViewBuilder
    func ifLet<Wrapped>(
        _ optional: Wrapped?,
        ifContent: (Self, Wrapped) -> some View,
        elseContent: (Self) -> some View,
    ) -> some View {
        if let value = optional {
            ifContent(self, value)
        } else {
            elseContent(self)
        }
    }
}

// MARK: - Sheet

public extension View {
    func sheet<T>(
        presenting data: Binding<T?>,
        sensoryFeedback: SensoryFeedback? = nil,
        onDismiss: (() -> Void)? = nil,
        @ViewBuilder content: @escaping (T) -> some View,
    ) -> some View {
        let isPresented = Binding<Bool>(
            get: { data.wrappedValue != nil },
            set: { newValue in
                guard !newValue else { return }
                data.wrappedValue = nil
            },
        )

        return sheet(
            isPresented: isPresented,
            onDismiss: onDismiss,
            content: {
                data.wrappedValue.map(content)
            },
        )
        .ifLet(sensoryFeedback) { view, value in
            view.sensoryFeedback(value, trigger: isPresented.wrappedValue) { $1 }
        }
    }
}

// MARK: - Syntactic sugar

public extension View {
    func frame(size: CGFloat, alignment: Alignment = .center) -> some View {
        frame(width: size, height: size, alignment: alignment)
    }
}

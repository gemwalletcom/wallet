// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI

public struct StateButtonStyle: ButtonStyle {
    public static let maxHeight: CGFloat = 50
    private let state: ButtonState
    private let palette: ButtonStylePalette

    public init(_ state: ButtonState, palette: ButtonStylePalette) {
        self.state = state
        self.palette = palette
    }

    public func makeBody(configuration: Configuration) -> some View {
        content(configuration: configuration)
            .frame(maxWidth: .infinity, maxHeight: Self.maxHeight)
            .background(adoptiveShape(configuration: configuration))
    }

    @ViewBuilder
    private func content(configuration: Configuration) -> some View {
        if state.showProgress {
            ButtonProgressView()
        } else {
            configuration.label
                .lineLimit(1)
                .foregroundStyle(foreground(configuration: configuration))
                .padding(.horizontal, .medium)
        }
    }

    @ViewBuilder
    private func adoptiveShape(configuration: Configuration) -> some View {
        if #available(iOS 26, *) {
            DefaultGlassEffectShape()
                .fill(background(configuration: configuration))
                .frame(maxHeight: Self.maxHeight)
                .glassEffect(.regular.interactive(!state.isDisabled))
        } else {
            RoundedRectangle(cornerRadius: Sizing.space12)
                .fill(background(configuration: configuration))
                .frame(maxHeight: Self.maxHeight)
        }
    }

    private func background(configuration: Configuration) -> Color {
        switch state {
        case .normal: configuration.isPressed ? palette.backgroundPressed : palette.background
        case let .loading(show): show ? palette.background : palette.backgroundDisabled
        case .disabled: palette.backgroundDisabled
        }
    }

    private func foreground(configuration: Configuration) -> Color {
        switch state {
        case .normal: configuration.isPressed ? palette.foregroundPressed : palette.foreground
        case let .loading(show): show ? palette.foreground : palette.foreground.opacity(.semi)
        case .disabled: palette.foreground
        }
    }
}

// MARK: - ButtonStyle Static

public extension ButtonStyle where Self == StateButtonStyle {
    static func primary(_ state: ButtonState = .normal) -> Self {
        .init(state, palette: .primary)
    }
}

// MARK: - Previews

#Preview {
    List {
        Section("Helpers .primary") {
            Button("Primary · normal") {}
                .buttonStyle(.primary())
            Button("Primary · loading") {}
                .buttonStyle(.primary(.loading()))
            Button("Primary · disabled") {}
                .buttonStyle(.primary(.disabled))
        }
    }
    .padding()
}

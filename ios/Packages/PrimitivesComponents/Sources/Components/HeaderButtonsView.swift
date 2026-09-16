// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import enum Gemstone.GemHeaderButtonKind
import Primitives
import SwiftUI

public typealias HeaderButtonAction = @MainActor @Sendable (GemHeaderButtonKind) -> Void

public struct HeaderButtonsView: View {
    private let buttons: [HeaderButton]
    private var action: HeaderButtonAction?

    var maxWidth: CGFloat {
        buttons.count > 3 ? 84 : 94
    }

    public init(
        buttons: [HeaderButton],
        action: HeaderButtonAction? = nil,
    ) {
        self.buttons = buttons
        self.action = action
    }

    public var body: some View {
        HStack(alignment: .center, spacing: .extraSmall) {
            ForEach(buttons) {
                buttonView(for: $0)
            }
        }
    }

    private func buttonView(for button: HeaderButton) -> some View {
        Group {
            switch button.viewType {
            case .button:
                RoundButton(
                    title: button.title,
                    image: button.image,
                    isEnabled: button.isEnabled,
                ) {
                    action?(button.type)
                }
            case let .menuButton(title, items):
                AdaptiveActionMenu(
                    title: title,
                    items: items,
                    label: {
                        RoundButton(
                            title: button.title,
                            image: button.image,
                            isEnabled: button.isEnabled,
                            action: {}, // action empty, handled by menu
                        )
                    },
                )
            }
        }
        .accessibilityIdentifier(button.id)
        .frame(maxWidth: maxWidth, alignment: .center)
    }
}

// MARK: - Previews

#Preview {
    let kinds: [GemHeaderButtonKind] = [.send, .receive, .buy, .swap, .deposit, .withdraw, .more]
    let buttons = kinds.map { HeaderButton(type: $0, isEnabled: true) }
    VStack {
        Spacer()
        HeaderButtonsView(buttons: buttons, action: nil)
        Spacer()
    }
}

// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemHeaderButton
import enum Gemstone.GemHeaderButtonTap
import Primitives
import SwiftUI

public typealias HeaderButtonAction = @MainActor @Sendable (GemHeaderButtonTap) -> Void

public struct HeaderButtonsView: View {
    private let buttons: [GemHeaderButton]
    private let menuTitle: String?
    private let menuItems: [ActionMenuItemType]
    private var action: HeaderButtonAction?

    var maxWidth: CGFloat {
        buttons.count > 3 ? 84 : 94
    }

    public init(
        buttons: [GemHeaderButton],
        menuTitle: String? = nil,
        menuItems: [ActionMenuItemType] = [],
        action: HeaderButtonAction? = nil,
    ) {
        self.buttons = buttons
        self.menuTitle = menuTitle
        self.menuItems = menuItems
        self.action = action
    }

    public var body: some View {
        HStack(alignment: .center, spacing: .extraSmall) {
            ForEach(buttons, id: \.self) {
                buttonView(for: $0)
            }
        }
    }

    private func buttonView(for button: GemHeaderButton) -> some View {
        Group {
            switch button.tap {
            case .send, .receive, .buy, .swap, .deposit, .withdraw, .sendCollectible:
                RoundButton(
                    title: button.kind.title,
                    image: button.kind.image,
                    isEnabled: button.isEnabled,
                ) {
                    action?(button.tap)
                }
            case .collectibleMenu:
                AdaptiveActionMenu(
                    title: menuTitle,
                    items: menuItems,
                    label: {
                        RoundButton(
                            title: button.kind.title,
                            image: button.kind.image,
                            isEnabled: button.isEnabled,
                            action: {}, // action empty, handled by menu
                        )
                    },
                )
            }
        }
        .accessibilityIdentifier("\(button.kind)_button")
        .frame(maxWidth: maxWidth, alignment: .center)
    }
}

// MARK: - Previews

#Preview {
    let buttons = [
        GemHeaderButton(kind: .send, tap: .send(assetId: nil), isEnabled: true),
        GemHeaderButton(kind: .receive, tap: .receive(assetId: nil), isEnabled: true),
        GemHeaderButton(kind: .buy, tap: .buy(assetId: nil), isEnabled: true),
        GemHeaderButton(kind: .swap, tap: .swap(payAssetId: nil, receiveAssetId: nil), isEnabled: true),
    ]
    VStack {
        Spacer()
        HeaderButtonsView(buttons: buttons, action: nil)
        Spacer()
    }
}

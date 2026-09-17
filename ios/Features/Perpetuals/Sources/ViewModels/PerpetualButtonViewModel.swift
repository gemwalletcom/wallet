// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemPerpetualButton

public struct PerpetualButtonViewModel: Identifiable, Hashable {
    public enum Style {
        case green
        case red
        case blue
    }

    let button: GemPerpetualButton

    public var id: String {
        String(describing: button)
    }

    public var title: String {
        button.title
    }

    public var isDestructive: Bool {
        button == .reduce
    }

    public var style: Style {
        switch button {
        case .long: .green
        case .short, .close: .red
        case .modify, .increase, .reduce: .blue
        }
    }
}

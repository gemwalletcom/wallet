// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import struct Gemstone.GemFormattedNumber
import enum Gemstone.GemHeaderActions
import enum Gemstone.GemLocalizedText
import enum Gemstone.GemValueTone
import GemstonePrimitives
import Primitives
import Style
import SwiftUI

public struct WalletHeaderViewModel {
    private let total: GemFormattedNumber
    private let pnl: GemLocalizedText?
    private let pnlTone: GemValueTone
    private let actions: GemHeaderActions

    public init(
        total: GemFormattedNumber,
        pnl: GemLocalizedText?,
        pnlTone: GemValueTone,
        actions: GemHeaderActions,
    ) {
        self.total = total
        self.pnl = pnl
        self.pnlTone = pnlTone
        self.actions = actions
    }
}

// MARK: - ValueHeaderViewModel

extension WalletHeaderViewModel: ValueHeaderViewModel {
    public var isWatchWallet: Bool {
        actions == .watchOnly
    }

    public var title: String {
        total.text()
    }

    public var assetImage: AssetImage? {
        .none
    }

    public var subtitle: String? {
        pnl?.text
    }

    public var subtitleColor: Color {
        pnlTone.color
    }

    public var subtitleImage: Image? {
        Image(systemName: SystemImage.chartLineUptrendXyaxis)
    }

    public var buttons: [HeaderButton] {
        switch actions {
        case .watchOnly: []
        case let .buttons(buttons): buttons.map { HeaderButton(type: $0.kind, isEnabled: $0.isEnabled) }
        }
    }
}

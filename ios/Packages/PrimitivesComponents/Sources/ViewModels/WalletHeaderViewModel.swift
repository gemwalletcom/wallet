// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import struct Gemstone.GemWalletHomeViewState
import GemstonePrimitives
import Primitives
import Style
import SwiftUI

public struct WalletHeaderViewModel {
    private let state: GemWalletHomeViewState

    public init(state: GemWalletHomeViewState) {
        self.state = state
    }
}

// MARK: - ValueHeaderViewModel

extension WalletHeaderViewModel: ValueHeaderViewModel {
    public var isWatchWallet: Bool {
        state.headerActions.isWatchOnly
    }

    public var title: String {
        state.total.text()
    }

    public var assetImage: AssetImage? {
        .none
    }

    public var subtitle: String? {
        state.pnl?.text
    }

    public var subtitleColor: Color {
        state.pnlTone.color
    }

    public var subtitleImage: Image? {
        Image(systemName: SystemImage.chartLineUptrendXyaxis)
    }

    public var buttons: [HeaderButton] {
        state.headerActions.headerButtons
    }
}

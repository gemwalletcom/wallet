// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import struct Gemstone.GemHeaderButton
import Primitives
import Style
import SwiftUI

public struct WalletHeaderViewModel {
    private let walletType: WalletType
    private let totalValue: TotalFiatValue
    private let headerButtons: [GemHeaderButton]
    private let totalValueViewModel: TotalValueViewModel

    public init(
        walletType: WalletType,
        totalValue: TotalFiatValue,
        currencyCode: String,
        showsPnl: Bool,
        buttons: [GemHeaderButton],
    ) {
        self.walletType = walletType
        self.totalValue = totalValue
        headerButtons = buttons
        let formatter = CurrencyFormatter(type: .fiat, currencyCode: currencyCode)
        totalValueViewModel = TotalValueViewModel(totalValue: totalValue, currencyFormatter: formatter, showsPnl: showsPnl)
    }
}

// MARK: - ValueHeaderViewModel

extension WalletHeaderViewModel: ValueHeaderViewModel {
    public var isWatchWallet: Bool {
        walletType == .view
    }

    public var title: String {
        totalValueViewModel.title
    }

    public var assetImage: AssetImage? {
        .none
    }

    public var subtitle: String? {
        guard let amount = totalValueViewModel.pnlAmountText else { return nil }
        guard let percentage = totalValueViewModel.pnlPercentageText else { return amount }
        return "\(amount) (\(percentage))"
    }

    public var subtitleColor: Color {
        totalValueViewModel.pnlColor
    }

    public var subtitleImage: Image? {
        Image(systemName: SystemImage.chartLineUptrendXyaxis)
    }

    public var buttons: [HeaderButton] {
        headerButtons.map { HeaderButton(type: $0.kind.headerButtonType, isEnabled: $0.isEnabled) }
    }
}

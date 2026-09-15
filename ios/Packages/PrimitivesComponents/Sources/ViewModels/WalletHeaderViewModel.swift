// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import GemstonePrimitives
import enum Gemstone.GemHeaderActions
import Primitives
import Style
import SwiftUI

public struct WalletHeaderViewModel {
    private let totalValue: TotalFiatValue
    private let actions: GemHeaderActions
    private let totalValueViewModel: TotalValueViewModel

    public init(
        totalValue: TotalFiatValue,
        currency: Currency,
        showsPnl: Bool,
        actions: GemHeaderActions,
    ) {
        self.totalValue = totalValue
        self.actions = actions
        let formatter = CurrencyFormatter(type: .fiat, currencyCode: currency.rawValue)
        totalValueViewModel = TotalValueViewModel(totalValue: totalValue, currencyFormatter: formatter, showsPnl: showsPnl)
    }
}

// MARK: - ValueHeaderViewModel

extension WalletHeaderViewModel: ValueHeaderViewModel {
    public var isWatchWallet: Bool {
        actions == .watchOnly
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
        switch actions {
        case .watchOnly: []
        case let .buttons(buttons): buttons.map { HeaderButton(type: $0.kind.headerButtonType, isEnabled: $0.isEnabled) }
        }
    }
}

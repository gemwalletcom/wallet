// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Foundation
import func Gemstone.delegationListRows
import struct Gemstone.GemDelegationListRow
import GemstonePrimitives
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

public struct DelegationViewModel: Sendable {
    private let row: GemDelegationListRow

    init(row: GemDelegationListRow) {
        self.row = row
    }

    static func items(_ delegations: [Delegation], asset: Asset, price: Double?, currency: Currency) -> [(delegation: Delegation, model: DelegationViewModel)] {
        let rows = delegationListRows(delegations: delegations.map { $0.toGem() }, asset: asset.toGem(), price: price, currency: currency.toGem())
        return zip(delegations, rows).map { delegation, row in
            (delegation, DelegationViewModel(row: row))
        }
    }

    public var listItem: ListItemModel {
        ListItemModel(
            title: validatorText,
            titleStyle: titleStyle,
            titleExtra: row.status.state.title,
            titleStyleExtra: TextStyle(font: .callout, color: row.status.tone.color),
            subtitle: balanceText,
            subtitleStyle: subtitleStyle,
            subtitleExtra: fiatValueText,
            subtitleStyleExtra: subtitleExtraStyle,
            imageStyle: .asset(assetImage: validatorImage),
        )
    }

    public var titleStyle: TextStyle {
        TextStyle(font: .body, color: .primary, fontWeight: .semibold)
    }

    public var subtitleStyle: TextStyle {
        TextStyle(font: .callout, color: row.balance.tone.color, fontWeight: .semibold)
    }

    public var subtitleExtraStyle: TextStyle {
        TextStyle(font: .footnote, color: Colors.gray)
    }

    public var balanceText: String {
        row.balance.text()
    }

    public var fiatValueText: String? {
        row.fiat?.text()
    }

    public var validatorText: String {
        ValidatorViewModel(row: row.validator).name
    }

    public var validatorImage: AssetImage {
        ValidatorViewModel(row: row.validator).validatorImage
    }
}

// MARK: - ValueHeaderViewModel

extension DelegationViewModel: ValueHeaderViewModel {
    public var isWatchWallet: Bool {
        false
    }

    public var buttons: [HeaderButton] {
        []
    }

    public var assetImage: AssetImage? {
        validatorImage
    }

    public var title: String {
        balanceText
    }

    public var subtitle: String? {
        fiatValueText
    }

    public var subtitleColor: Color {
        .secondary
    }
}

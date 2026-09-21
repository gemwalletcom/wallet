// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Foundation
import func Gemstone.delegationListRow
import struct Gemstone.GemDelegationListRow
import struct Gemstone.GemDelegationStatus
import protocol Gemstone.GemStakeServiceProtocol
import GemstonePrimitives
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

public struct DelegationViewModel: Sendable {
    public let delegation: Delegation
    public let currency: Currency
    private let row: GemDelegationListRow
    public let validatorModel: ValidatorViewModel

    public init(
        service _: any GemStakeServiceProtocol,
        delegation: Delegation,
        asset: Asset,
        formatter _: ValueFormatter = .short,
        currency: Currency,
    ) {
        self.delegation = delegation
        self.currency = currency
        row = delegationListRow(
            delegation: delegation.toGem(),
            asset: asset.toGem(),
            price: delegation.price?.price,
            currency: currency.toGem(),
        )
        validatorModel = ValidatorViewModel(row: row.validator)
    }

    public var status: GemDelegationStatus {
        row.status
    }

    public var listItem: ListItemModel {
        ListItemModel(
            title: validatorText,
            titleStyle: titleStyle,
            titleExtra: status.state.title,
            titleStyleExtra: TextStyle(font: .callout, color: status.tone.color),
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
        TextStyle(font: .callout, color: row.hasBalance ? Colors.black : Colors.gray, fontWeight: .semibold)
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

    public var rewardsText: String? {
        row.rewards?.text()
    }

    public var rewardsFiatValueText: String? {
        row.rewardsFiat?.text()
    }

    public var validatorText: String {
        validatorModel.name
    }

    public var validatorImage: AssetImage {
        validatorModel.validatorImage
    }
}

extension DelegationViewModel: Identifiable {
    public var id: String {
        delegation.id
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

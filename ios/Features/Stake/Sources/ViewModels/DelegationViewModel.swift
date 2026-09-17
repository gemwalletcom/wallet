// Copyright (c). Gem Wallet. All rights reserved.

import Components
import func Gemstone.delegationStatus
import struct Gemstone.GemDelegationStatus
import protocol Gemstone.GemStakeServiceProtocol
import GemstonePrimitives
import Formatters
import Foundation
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

public struct DelegationViewModel: Sendable {
    public let delegation: Delegation
    public let currency: Currency
    private let asset: Asset
    private let formatter: ValueFormatter
    private let service: any GemStakeServiceProtocol
    private let priceViewModel: PriceViewModel
    public let validatorModel: ValidatorViewModel

    public init(
        service: any GemStakeServiceProtocol,
        delegation: Delegation,
        asset: Asset,
        formatter: ValueFormatter = .short,
        currency: Currency,
    ) {
        self.delegation = delegation
        self.currency = currency
        self.asset = asset
        self.formatter = formatter
        self.service = service
        priceViewModel = PriceViewModel(price: delegation.price, currencyCode: currency.rawValue)
        validatorModel = ValidatorViewModel(row: service.validatorRow(validator: delegation.validator.toGem()))
    }

    public var status: GemDelegationStatus {
        delegationStatus(delegation: delegation.toGem())
    }

    public var listItem: ListItemModel {
        ListItemModel(
            title: validatorText,
            titleStyle: titleStyle,
            titleExtra: stateModel.title,
            titleStyleExtra: stateModel.textStyle,
            subtitle: balanceText,
            subtitleStyle: subtitleStyle,
            subtitleExtra: fiatValueText,
            subtitleStyleExtra: subtitleExtraStyle,
            imageStyle: .asset(assetImage: validatorImage),
        )
    }

    public var stateModel: DelegationStateViewModel {
        DelegationStateViewModel(status: status)
    }

    public var titleStyle: TextStyle {
        TextStyle(font: .body, color: .primary, fontWeight: .semibold)
    }

    public var subtitleStyle: TextStyle {
        TextStyle(font: .callout, color: Colors.black, fontWeight: .semibold)
    }

    public var subtitleExtraStyle: TextStyle {
        TextStyle(font: .footnote, color: Colors.gray)
    }

    public var balanceText: String {
        formatter.string(delegation.base.balance, decimals: asset.decimals.asInt, currency: asset.symbol)
    }

    public var fiatValueText: String? {
        priceViewModel.fiatValueText(value: delegation.base.balance, decimals: asset.decimals.asInt)
    }

    private var showsRewards: Bool {
        service.showsRewards(delegation: delegation.base.toGem())
    }

    public var rewardsText: String? {
        guard showsRewards else { return nil }
        return formatter.string(delegation.base.rewards, decimals: asset.decimals.asInt, currency: asset.symbol)
    }

    public var rewardsFiatValueText: String? {
        guard showsRewards else { return nil }
        return priceViewModel.fiatValueText(value: delegation.base.rewards, decimals: asset.decimals.asInt)
    }

    public var validatorText: String {
        validatorModel.name
    }

    public var validatorImage: AssetImage {
        validatorModel.validatorImage
    }

    public var validatorUrl: URL? {
        service.validatorUrl(validator: delegation.validator.toGem()).map { $0.toPrimitives() }?.url
    }

    public var completionDateText: String? {
        guard
            status.completion != nil,
            let completionDate = delegation.base.completionDate,
            case let remaining = Date.now.distance(to: completionDate),
            remaining > 0
        else { return nil }
        return CountdownFormatter().string(seconds: Int64(remaining))
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

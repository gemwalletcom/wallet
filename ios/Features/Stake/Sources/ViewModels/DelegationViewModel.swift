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
    public let currencyCode: String
    private let asset: Asset
    private let formatter: ValueFormatter
    private let service: any GemStakeServiceProtocol
    private let priceFormatter: CurrencyFormatter

    public init(
        service: any GemStakeServiceProtocol,
        delegation: Delegation,
        asset: Asset,
        formatter: ValueFormatter = .short,
        currencyCode: String,
    ) {
        self.delegation = delegation
        self.currencyCode = currencyCode
        self.asset = asset
        self.formatter = formatter
        self.service = service
        priceFormatter = CurrencyFormatter(type: .currency, currencyCode: currencyCode)
    }

    public var status: GemDelegationStatus {
        delegationStatus(delegation: delegation.map())
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
        guard
            let price = delegation.price,
            let balance = try? formatter.double(from: delegation.base.balance, decimals: asset.decimals.asInt)
        else { return nil }
        return priceFormatter.string(price.price * balance)
    }

    private var showsRewards: Bool {
        service.showsRewards(delegation: delegation.base.map())
    }

    public var rewardsText: String? {
        guard showsRewards else { return nil }
        return formatter.string(delegation.base.rewards, decimals: asset.decimals.asInt, currency: asset.symbol)
    }

    public var rewardsFiatValueText: String? {
        guard
            showsRewards,
            let price = delegation.price,
            let rewards = try? formatter.double(from: delegation.base.rewards, decimals: asset.decimals.asInt)
        else { return nil }
        return priceFormatter.string(price.price * rewards)
    }

    public var validatorModel: ValidatorViewModel {
        ValidatorViewModel(row: service.validatorRow(validator: delegation.validator.map()))
    }

    public var validatorText: String {
        validatorModel.name
    }

    public var validatorImage: AssetImage {
        validatorModel.validatorImage
    }

    public var validatorUrl: URL? {
        service.validatorUrl(validator: delegation.validator.map()).map { $0.map() }?.url
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

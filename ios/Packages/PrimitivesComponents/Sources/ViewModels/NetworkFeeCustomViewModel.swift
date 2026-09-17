// Copyright (c). Gem Wallet. All rights reserved.

import Components
import BigInt
import Formatters
import Foundation
import class Gemstone.GemCustomFee
import struct Gemstone.GemNumberFormat
import GemstonePrimitives
import Localization
import Observation
import Primitives

@Observable
@MainActor
public final class NetworkFeeCustomViewModel {
    private let chain: Chain
    private let feeAsset: Asset
    private let unitType: FeeUnitType
    private let baseFee: BigInt?
    private let baseTotal: BigInt?
    private let normalTotal: BigInt?
    private let decimals: Int
    private let onSelect: @MainActor (BigInt) -> Void
    private let display: (BigInt) -> AmountDisplay

    public var input: String = ""

    public init(
        chain: Chain,
        feeAsset: Asset,
        unitType: FeeUnitType,
        decimals: Int,
        baseFee: BigInt?,
        baseTotal: BigInt?,
        normalTotal: BigInt?,
        initialRate: BigInt?,
        onSelect: @escaping @MainActor (BigInt) -> Void,
        display: @escaping (BigInt) -> AmountDisplay,
    ) {
        self.chain = chain
        self.feeAsset = feeAsset
        self.unitType = unitType
        self.decimals = decimals
        self.baseFee = baseFee
        self.baseTotal = baseTotal
        self.normalTotal = normalTotal
        self.onSelect = onSelect
        self.display = display
        input = initialRate.map { ValueFormatter.full.string($0, decimals: decimals) } ?? ""
    }

    public var title: String { Localized.FeeRate.custom }
    public var networkFeeListItem: ListItemModel {
        ListItemModel(title: networkFeeTitle, subtitle: value, subtitleExtra: fiatValue)
    }

    public var networkFeeTitle: String { Localized.Transfer.networkFee }

    public var suffix: String {
        FeeUnitViewModel(unit: FeeUnit(type: unitType, value: .zero), decimals: decimals, symbol: feeAsset.symbol).suffix
    }

    public var placeholder: String {
        baseTotal.map { ValueFormatter.auto.string($0, decimals: decimals) } ?? ""
    }

    public var value: String? {
        feeAmount.map { display($0).amount.text }
    }

    public var fiatValue: String? {
        feeAmount.flatMap { display($0).fiat?.text }
    }

    public var errorText: String? {
        switch estimate.check() {
        case .belowMinimum: estimate.minimumRate().map { Localized.Common.minimumValue(rateText($0)) }
        case .overMaximum: Localized.Common.maximumValue(rateText(estimate.maxRate()))
        case .valid: nil
        }
    }

    private func rateText(_ rate: BigInt) -> String {
        FeeUnitViewModel(unit: FeeUnit(type: unitType, value: rate), decimals: decimals, symbol: feeAsset.symbol).value
    }

    public var isConfirmEnabled: Bool {
        estimate.isValid()
    }

    public func sanitize(_ text: String) -> String {
        NumberInput.format().sanitize(input: text, maximumFractionDigits: UInt32(decimals), maximumIntegerDigits: nil)
    }

    public func confirm() {
        guard let rate, estimate.isValid() else { return }
        onSelect(rate)
    }

    private var rate: BigInt? {
        guard let value = try? NumberInput.value(input, decimals: decimals), value > .zero else { return nil }
        return value
    }

    private var estimate: GemCustomFee {
        GemCustomFee.estimate(
            chain: chain.rawValue,
            rate: rate,
            loadedFee: baseFee ?? .zero,
            baseTotal: baseTotal ?? .zero,
            normalTotal: normalTotal ?? .zero,
        )
    }

    private var feeAmount: BigInt? {
        baseFee.map { _ in estimate.feeValue() }
    }
}

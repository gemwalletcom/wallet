// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Formatters
import class Gemstone.GemCustomFee
import GemstonePrimitives
import Localization
import Observation
import Primitives

@Observable
@MainActor
public final class NetworkFeeCustomViewModel {
    private let chain: Chain
    private let feeAsset: Asset
    private let feeAssetPrice: Price?
    private let currency: Currency
    private let unitType: FeeUnitType
    private let baseFee: BigInt?
    private let baseTotal: BigInt?
    private let normalTotal: BigInt?
    private let decimals: Int
    private let onSelect: @MainActor (BigInt) -> Void

    public var input: String = ""

    public init(
        chain: Chain,
        feeAsset: Asset,
        feeAssetPrice: Price?,
        currency: Currency,
        unitType: FeeUnitType,
        decimals: Int,
        baseFee: BigInt?,
        baseTotal: BigInt?,
        normalTotal: BigInt?,
        initialRate: BigInt?,
        onSelect: @escaping @MainActor (BigInt) -> Void,
    ) {
        self.chain = chain
        self.feeAsset = feeAsset
        self.feeAssetPrice = feeAssetPrice
        self.currency = currency
        self.unitType = unitType
        self.decimals = decimals
        self.baseFee = baseFee
        self.baseTotal = baseTotal
        self.normalTotal = normalTotal
        self.onSelect = onSelect
        input = initialRate.map { ValueFormatter.full.string($0, decimals: decimals) } ?? ""
    }

    public var title: String { Localized.FeeRate.custom }
    public var networkFeeTitle: String { Localized.Transfer.networkFee }

    public var suffix: String {
        FeeUnitViewModel(unit: FeeUnit(type: unitType, value: .zero), decimals: decimals, symbol: feeAsset.symbol).suffix
    }

    public var placeholder: String {
        baseTotal.map { ValueFormatter.auto.string($0, decimals: decimals) } ?? ""
    }

    public var value: String? {
        feeAmount.map { display(for: $0).amount.text }
    }

    public var fiatValue: String? {
        feeAmount.flatMap { display(for: $0).fiat?.text }
    }

    public var errorText: String? {
        if estimate.isBelowMinimum(), let minimumRate = estimate.minimumRate() {
            let minText = FeeUnitViewModel(unit: FeeUnit(type: unitType, value: minimumRate), decimals: decimals, symbol: feeAsset.symbol).value
            return Localized.Common.minimumValue(minText)
        }
        if estimate.isOverMax() {
            let maxText = FeeUnitViewModel(unit: FeeUnit(type: unitType, value: estimate.maxRate()), decimals: decimals, symbol: feeAsset.symbol).value
            return Localized.Common.maximumValue(maxText)
        }
        return nil
    }

    public var isConfirmEnabled: Bool {
        estimate.isValid()
    }

    public func sanitize(_ text: String) -> String {
        NumberSanitizer(maximumFractionDigits: decimals).sanitize(text)
    }

    public func confirm() {
        guard let rate, estimate.isValid() else { return }
        onSelect(rate)
    }

    private var rate: BigInt? {
        guard let value = try? ValueFormatter.full.inputNumber(from: input, decimals: decimals), value > .zero else { return nil }
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

    private func display(for amount: BigInt) -> AmountDisplay {
        AmountDisplay.numeric(
            asset: feeAsset,
            price: feeAssetPrice,
            value: amount,
            currency: currency.rawValue,
            formatter: .auto,
        )
    }
}

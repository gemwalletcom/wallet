// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Formatters
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
    private let baseFee: BigInt?
    private let baseTotal: BigInt?
    private let onSelect: (BigInt) -> Void

    public var input: String = ""

    public init(
        chain: Chain,
        feeAsset: Asset,
        feeAssetPrice: Price?,
        currency: Currency,
        baseFee: BigInt?,
        baseTotal: BigInt?,
        initialRate: BigInt?,
        onSelect: @escaping (BigInt) -> Void,
    ) {
        self.chain = chain
        self.feeAsset = feeAsset
        self.feeAssetPrice = feeAssetPrice
        self.currency = currency
        self.baseFee = baseFee
        self.baseTotal = baseTotal
        self.onSelect = onSelect
        input = initialRate.map { ValueFormatter.full.string($0, decimals: chain.feeUnitDecimals) } ?? ""
    }

    public var title: String {
        Localized.FeeRate.custom
    }

    public var networkFeeTitle: String {
        Localized.Transfer.networkFee
    }

    public var suffix: String {
        FeeUnitViewModel(unit: FeeUnit(type: chain.feeUnitType, value: .zero), decimals: decimals, symbol: feeAsset.symbol).suffix
    }

    public var placeholder: String {
        baseTotal.map { ValueFormatter.full.string($0, decimals: decimals) } ?? ""
    }

    public var value: String? {
        feeAmount.map { display(for: $0).amount.text }
    }

    public var fiatValue: String? {
        feeAmount.flatMap { display(for: $0).fiat?.text }
    }

    public var errorText: String? {
        guard isAboveMax, let maxRate else { return nil }
        let maxText = FeeUnitViewModel(unit: FeeUnit(type: chain.feeUnitType, value: maxRate), decimals: decimals, symbol: feeAsset.symbol).value
        return Localized.Common.maximumValue(maxText)
    }

    public var isConfirmEnabled: Bool {
        rate != nil && isAboveMax == false
    }

    public func sanitize(_ text: String) -> String {
        NumberSanitizer(maximumFractionDigits: decimals).sanitize(text)
    }

    public func confirm() {
        guard isConfirmEnabled, let rate else { return }
        onSelect(rate)
    }

    private var decimals: Int {
        chain.feeUnitDecimals
    }

    private var rate: BigInt? {
        guard let value = try? ValueFormatter.full.inputNumber(from: input, decimals: decimals), value > .zero else { return nil }
        return value
    }

    private var maxRate: BigInt? {
        baseTotal.map { $0 * BigInt(chain.maxCustomFeeRateMultiplier) }
    }

    private var isAboveMax: Bool {
        guard let rate, let maxRate else { return false }
        return rate > maxRate
    }

    private var feeAmount: BigInt? {
        guard let baseFee, let baseTotal, baseTotal != .zero, let rate else { return baseFee }
        return baseFee * rate / baseTotal
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

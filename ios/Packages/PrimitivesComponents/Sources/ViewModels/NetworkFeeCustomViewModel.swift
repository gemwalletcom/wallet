// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Formatters
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
    private let baseFee: BigInt?
    private let baseTotal: BigInt?
    private let normalTotal: BigInt?
    private let decimals: Int
    private let onSelect: (BigInt) -> Void

    public var input: String = ""

    public init(
        chain: Chain,
        feeAsset: Asset,
        feeAssetPrice: Price?,
        currency: Currency,
        baseFee: BigInt?,
        baseTotal: BigInt?,
        normalTotal: BigInt?,
        initialRate: BigInt?,
        onSelect: @escaping (BigInt) -> Void,
    ) {
        self.chain = chain
        self.feeAsset = feeAsset
        self.feeAssetPrice = feeAssetPrice
        self.currency = currency
        self.baseFee = baseFee
        self.baseTotal = baseTotal
        self.normalTotal = normalTotal
        self.onSelect = onSelect
        decimals = switch chain.feeUnitType {
        case .satVb, .gwei: chain.feeUnitDecimals
        case .native: feeAsset.decimals.asInt
        }
        input = initialRate.map { ValueFormatter.full.string($0, decimals: decimals) } ?? ""
    }

    public var title: String { Localized.FeeRate.custom }
    public var networkFeeTitle: String { Localized.Transfer.networkFee }

    public var suffix: String {
        FeeUnitViewModel(unit: FeeUnit(type: chain.feeUnitType, value: .zero), decimals: decimals, symbol: feeAsset.symbol).suffix
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
        guard let estimate, estimate.isOverMax else { return nil }
        let maxText = FeeUnitViewModel(unit: FeeUnit(type: chain.feeUnitType, value: estimate.maxRate), decimals: decimals, symbol: feeAsset.symbol).value
        return Localized.Common.maximumValue(maxText)
    }

    public var isConfirmEnabled: Bool {
        rate != nil && estimate?.isOverMax == false
    }

    public func sanitize(_ text: String) -> String {
        NumberSanitizer(maximumFractionDigits: decimals).sanitize(text)
    }

    public func confirm() {
        guard let rate, estimate?.isOverMax == false else { return }
        onSelect(rate)
    }

    private var rate: BigInt? {
        guard let value = try? ValueFormatter.full.inputNumber(from: input, decimals: decimals), value > .zero else { return nil }
        return value
    }

    private var estimate: CustomFeeEstimate? {
        try? CustomFeeEstimate.estimate(
            rate: rate,
            loadedFee: baseFee ?? .zero,
            baseTotal: baseTotal ?? .zero,
            normalTotal: normalTotal ?? .zero,
            maxMultiplier: chain.maxCustomFeeRateMultiplier,
        )
    }

    private var feeAmount: BigInt? {
        baseFee.flatMap { _ in estimate?.feeAmount }
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

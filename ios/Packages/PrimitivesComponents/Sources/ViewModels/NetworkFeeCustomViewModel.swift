// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Components
import Formatters
import Foundation
import func Gemstone.customFeeEstimate
import struct Gemstone.GemCustomFeeEstimate
import struct Gemstone.GemCustomFeeInput
import struct Gemstone.GemFeeRateRows
import GemstonePrimitives
import Localization
import Observation
import Primitives

@Observable
@MainActor
public final class NetworkFeeCustomViewModel {
    private let feeAsset: Asset
    private let rows: GemFeeRateRows
    private let baseFee: BigInt?
    private let price: Double?
    private let currency: Currency
    private let onSelect: @MainActor (BigInt) -> Void

    public var input: String {
        didSet { estimate = Self.estimate(input: input, feeAsset: feeAsset, rows: rows, baseFee: baseFee, price: price, currency: currency) }
    }

    private var estimate: GemCustomFeeEstimate

    public init(
        feeAsset: Asset,
        rows: GemFeeRateRows,
        baseFee: BigInt?,
        initialRate: BigInt?,
        price: Double?,
        currency: Currency,
        onSelect: @escaping @MainActor (BigInt) -> Void,
    ) {
        self.feeAsset = feeAsset
        self.rows = rows
        self.baseFee = baseFee
        self.price = price
        self.currency = currency
        self.onSelect = onSelect
        let input = initialRate.flatMap { NumberInput.format().inputText(value: $0.description, decimals: rows.unitDecimals) } ?? ""
        self.input = input
        estimate = Self.estimate(input: input, feeAsset: feeAsset, rows: rows, baseFee: baseFee, price: price, currency: currency)
    }

    public var title: String { Localized.FeeRate.custom }
    public var networkFeeListItem: ListItemModel {
        ListItemModel(title: networkFeeTitle, subtitle: value, subtitleExtra: fiatValue)
    }

    public var networkFeeTitle: String { Localized.Transfer.networkFee }

    public var suffix: String {
        rows.unitType.toPrimitives().suffix(symbol: feeAsset.symbol)
    }

    public var placeholder: String {
        estimate.placeholder?.text() ?? ""
    }

    public var value: String? {
        estimate.fee?.amount.text()
    }

    public var fiatValue: String? {
        estimate.fee?.fiat?.text()
    }

    public var errorText: String? {
        switch estimate.check {
        case let .belowMinimum(rate): Localized.Common.minimumValue(rate.text)
        case let .overMaximum(rate): Localized.Common.maximumValue(rate.text)
        case .valid: nil
        }
    }

    public var isConfirmEnabled: Bool {
        estimate.isValid
    }

    public func sanitize(_ text: String) -> String {
        NumberInput.format().sanitize(input: text, maximumFractionDigits: rows.unitDecimals, maximumIntegerDigits: nil)
    }

    public func confirm() {
        guard let rate = estimate.rate, estimate.isValid else { return }
        onSelect(rate)
    }

    private static func estimate(input: String, feeAsset: Asset, rows: GemFeeRateRows, baseFee: BigInt?, price: Double?, currency: Currency) -> GemCustomFeeEstimate {
        customFeeEstimate(input: GemCustomFeeInput(
            feeAsset: feeAsset.toGem(),
            input: input,
            format: NumberInput.format(),
            rows: rows,
            loadedFee: baseFee,
            price: price,
            currency: currency.toGem(),
        ))
    }
}
